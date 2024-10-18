use anyhow::Result;
use anyhow::Error;
use blocksense_sdk::{
    oracle::{DataFeedResult, DataFeedResultValue, Payload, Settings},
    oracle_component,
    spin::http::{send, Method, Request, Response},
};
use serde::Deserialize;
use chrono::Duration;
use chrono::Utc;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ForecastResponse {
    daily: DailyData,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct DailyData {
    precipitation_sum: Vec<f64>,
    snowfall_sum: Vec<f64>,
    time: Vec<String>,
}

fn get_previous_day() -> String {
    // Get today's date in UTC
    let today = Utc::now().date_naive();

    // Subtract one day
    let previous_day = today - Duration::days(1);

    // Format the date as "YYYY-MM-DD"
    let formatted_date = previous_day.format("%Y-%m-%d").to_string();

    // Return the start and end date as the same day
    formatted_date
}

async fn get_forecast(start_date: &str) -> Result<ForecastResponse, Error> {
    let url = format!(
        "https://historical-forecast-api.open-meteo.com/v1/forecast?latitude=42.698334&longitude=23.319941&start_date={}&end_date={}&daily=precipitation_sum,snowfall_sum&timezone=Europe%2FBerlin",
        start_date, start_date
    );

    let mut req = Request::builder();
    req.method(Method::Get);
    req.uri(url);
    req.header("user-agent", "*/*");
    req.header("Accepts", "application/json");

    let req = req.build();
    let resp: Response = send(req).await?;

    let body = resp.into_body();
    let string = String::from_utf8(body).expect("Our bytes should be valid utf8");
    let value: ForecastResponse = serde_json::from_str(&string).unwrap();

    println!("{:?}", value);

    Ok(value)
}

#[oracle_component]
async fn oracle_request(settings: Settings) -> Result<Payload> {
    let mut payload: Payload = Payload::new();
    for data_feed in settings.data_feeds.iter() {
        // get today and subtract 1 day and format to this
        let start_date = get_previous_day();

        if data_feed.id == "31" {
            match get_forecast(&start_date).await {
                Ok(forecast) => {
                    println!("{:#?}", forecast);
                    // Check if there is at least one precipitation value in the array
                    if let Some(precipitation) = forecast.daily.precipitation_sum.get(0) {
                        // Push the value to the payload
                        payload.values.push(DataFeedResult {
                            id: "31".to_string(),
                            // value: DataFeedResultValue::Numerical(*precipitation),
                            value: DataFeedResultValue::Numerical(15.0)
                        });
                    } else {
                        eprintln!("No precipitation data available.");
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching forecast data: {}", e);
                }
            }
        } else {
            match get_forecast(&start_date).await {
                Ok(forecast) => {
                    println!("{:#?}", forecast);
                    // Check if there is at least one precipitation value in the array
                    if let Some(precipitation) = forecast.daily.snowfall_sum.get(0) {
                        // Push the value to the payload
                        payload.values.push(DataFeedResult {
                            id: "47".to_string(),
                            // value: DataFeedResultValue::Numerical(*precipitation),
                            value: DataFeedResultValue::Numerical(20.0)
                        });
                    } else {
                        eprintln!("No snowfall data available.");
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching forecast data: {}", e);
                }
            }
        }
    }
    Ok(payload)
}
