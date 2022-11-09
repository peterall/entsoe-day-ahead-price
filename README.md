# entsoe-day-ahead-price
A Rust library for fetching day-ahead electricity spot prices from [ENTSO-E Transparency](https://transparency.entsoe.eu) platform. Only Swedish regions supported for now.

Note that a security token is required which can be requested from ENTSO-E [here](https://transparency.entsoe.eu/content/static_content/download?path=/Static%20content/API-Token-Management.pdf).

## Example
```rust
let entsoe = Entsoe::new("my-security-token");
let prices = entsoe
    .get_day_ahead_prices("SE3", Stockholm.ymd(2022, 11, 9))
    .await
    .unwrap();

println!("{:26}{:8}", "Hour", "Price");
for price in prices.iter() {
    println!("{:24}{:>8}",
        price.start_time.with_timezone(&Stockholm),
        price.amount.to_string(),
    );
}
```

Output:
```
Hour                      Price   
2022-11-09 00:00:00 CET  €19,56
2022-11-09 01:00:00 CET  €18,77
2022-11-09 02:00:00 CET  €18,22
2022-11-09 03:00:00 CET  €17,48
2022-11-09 04:00:00 CET  €18,44
2022-11-09 05:00:00 CET  €19,84
2022-11-09 06:00:00 CET  €22,89
2022-11-09 07:00:00 CET  €29,45
2022-11-09 08:00:00 CET  €33,07
2022-11-09 09:00:00 CET  €35,03
2022-11-09 10:00:00 CET  €39,90
2022-11-09 11:00:00 CET  €38,25
2022-11-09 12:00:00 CET  €33,06
2022-11-09 13:00:00 CET  €33,09
2022-11-09 14:00:00 CET  €33,04
2022-11-09 15:00:00 CET  €48,10
2022-11-09 16:00:00 CET  €60,75
2022-11-09 17:00:00 CET  €61,92
2022-11-09 18:00:00 CET  €60,35
2022-11-09 19:00:00 CET  €35,07
2022-11-09 20:00:00 CET  €33,07
2022-11-09 21:00:00 CET  €30,95
2022-11-09 22:00:00 CET  €29,55
2022-11-09 23:00:00 CET  €26,83
```