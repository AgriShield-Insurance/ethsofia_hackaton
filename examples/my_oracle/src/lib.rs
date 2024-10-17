use anyhow::Result;
use anyhow::Error;
use blocksense_sdk::{
    oracle::{DataFeedResult, DataFeedResultValue, Payload, Settings},
    oracle_component,
    spin::http::{send, Method, Request, Response},
};
use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ForecastResponse {
    daily: DailyData,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct DailyData {
    precipitation_sum: Vec<f64>,
    time: Vec<String>,
}

async fn get_forecast(start_date: &str, end_date: &str) -> Result<ForecastResponse, Error> {
    let url = format!(
        "https://historical-forecast-api.open-meteo.com/v1/forecast?latitude=42.698334&longitude=23.319941&start_date={}&end_date={}&daily=precipitation_sum&timezone=Europe%2FBerlin",
        start_date, end_date
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

    let start_date = "2024-10-17";
    let end_date = "2024-10-17";

    match get_forecast(start_date, end_date).await {
        Ok(forecast) => {
            println!("Got a response");
            println!("{:#?}", forecast);
            // Check if there is at least one precipitation value in the array
            if let Some(precipitation) = forecast.daily.precipitation_sum.get(0) {
                // Push the value to the payload
                payload.values.push(DataFeedResult {
                    id: "SofiaPrecipitation".to_string(),
                    value: DataFeedResultValue::Numerical(*precipitation),
                });
            } else {
                eprintln!("No precipitation data available.");
            }
        }
        Err(e) => {
            eprintln!("Error fetching forecast data: {}", e);
        }
    }

    // for data_feed in settings.data_feeds.iter() {
    //     let url = Url::parse(format!("https://www.revolut.com/api/quote/public/{}", data_feed.data).as_str())?;
    //     println!("URL - {}", url.as_str());
        // let mut req = Request::builder();
        // req.method(Method::Get);
        // req.uri(url);
        // req.header("user-agent", "*/*");
        // req.header("Accepts", "application/json");

        // let req = req.build();
        // let resp: Response = send(req).await?;

        // let body = resp.into_body();
        // let string = String::from_utf8(body).expect("Our bytes should be valid utf8");
        // let value: Rate = serde_json::from_str(&string).unwrap();

        // println!("{:?}", value);

    //     payload.values.push(DataFeedResult {
    //         id: data_feed.id.clone(),
    //         value: DataFeedResultValue::Numerical(value.rate),
    //     });
    // }
    Ok(payload)
}
