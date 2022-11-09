use chrono::{Date, DateTime, Duration, TimeZone, Utc};
use chrono_tz::Tz;

use rusty_money::MoneyError;
use serde::{Deserialize, Deserializer};
use thiserror::Error;

type Money = rusty_money::Money<'static, rusty_money::iso::Currency>;

#[derive(Error, Debug)]
pub enum GetPriceError {
    #[error("the area `{0}` is not supported")]
    InvalidArea(String),
    #[error("error in entsoe request")]
    RequestError(#[from] reqwest::Error),
    #[error("error in response format")]
    ResponseFormat(#[from] serde_xml_rs::Error),
    #[error("the currenty `{0}` is not supported")]
    UnknownCurrency(String),
    #[error("the resolution `{0}` is not supported")]
    UnknownResolution(String),
    #[error("invalid money format")]
    UnknownMoneyFormat(#[from] MoneyError),
}
pub struct Entsoe {
    security_token: String,
}

impl Entsoe {
    pub fn new(security_token: &str) -> Self {
        Entsoe {
            security_token: security_token.to_owned(),
        }
    }
    pub async fn get_day_ahead_prices(
        &self,
        area: &str,
        date: Date<Tz>,
    ) -> Result<Vec<Price>, GetPriceError> {
        let security_token = &self.security_token;
        let domain = Self::get_domain(area)?;
        let date = date.format("%Y%m%d");

        let url = format!(
            "https://web-api.tp.entsoe.eu/api\
                ?securityToken={security_token}\
                &documentType=A44\
                &in_Domain={domain}\
                &out_Domain={domain}\
                &periodStart={date}0000\
                &periodEnd={date}2300"
        );

        let MarketDocument { time_series: ts } =
            serde_xml_rs::from_str(&reqwest::get(url).await?.text().await?)?;

        let currency = rusty_money::iso::find(&ts.currency)
            .ok_or_else(|| GetPriceError::UnknownCurrency(ts.currency))?;

        if ts.period.resolution != "PT60M" {
            Err(GetPriceError::UnknownResolution(ts.period.resolution))
        } else {
            ts.period
                .points
                .iter()
                .map(|p| {
                    Money::from_str(&p.price.replace('.', ","), currency)
                        .map_err(MoneyError::into)
                        .map(|amount| Price {
                            start_time: ts.period.time_interval.start
                                + Duration::hours(p.position - 1),
                            amount,
                        })
                })
                .collect()
        }
    }

    fn get_domain(area: &str) -> Result<&'static str, GetPriceError> {
        match area {
            "SE1" => Ok("10Y1001A1001A44P"),
            "SE2" => Ok("10Y1001A1001A45N"),
            "SE3" => Ok("10Y1001A1001A46L"),
            "SE4" => Ok("10Y1001A1001A47J"),
            _ => Err(GetPriceError::InvalidArea(area.to_owned())),
        }
    }
}

fn deserialize_timeinterval<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Utc.datetime_from_str(&s, "%Y-%m-%dT%H:%MZ")
        .map_err(serde::de::Error::custom)
}

#[derive(Deserialize, Debug)]
struct MarketDocument {
    #[serde(rename = "TimeSeries")]
    time_series: TimeSeries,
}

#[derive(Deserialize, Debug)]
struct TimeSeries {
    #[serde(rename = "currency_Unit.name")]
    currency: String,
    #[serde(rename = "Period")]
    period: Period,
}

#[derive(Deserialize, Debug)]
struct Period {
    #[serde(rename = "timeInterval")]
    time_interval: TimeInterval,
    #[serde(rename = "resolution")]
    resolution: String,
    #[serde(rename = "Point")]
    points: Vec<Point>,
}
#[derive(Deserialize, Debug)]
struct TimeInterval {
    #[serde(deserialize_with = "deserialize_timeinterval")]
    start: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
struct Point {
    position: i64,
    #[serde(rename = "price.amount")]
    price: String,
}

pub struct Price {
    pub start_time: DateTime<Utc>,
    pub amount: Money,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use chrono_tz::Europe::Stockholm;
    use std::fs;

    #[tokio::test]
    async fn get_day_ahead_prices() {
        let entsoe = Entsoe::new(&fs::read_to_string("security_token").unwrap());
        let prices = entsoe
            .get_day_ahead_prices("SE3", Stockholm.ymd(2022, 11, 9))
            .await
            .unwrap();

        assert_eq!(prices.len(), 24);

        println!("{:26}{:8}", "Hour", "Price");
        for price in prices.iter() {
            println!(
                "{:24}{:>8}",
                price.start_time.with_timezone(&Stockholm),
                price.amount.to_string(),
            );
        }
    }
}
