use std::{collections::HashMap, env};

use crate::tool::api_result::ApiError;
use phonenumber::{country, Mode};
use reqwest::header::{HeaderMap, CONTENT_TYPE};
use rocket::Route;
use serde::{Deserialize, Serialize};
use url::form_urlencoded;

pub fn endpoints() -> Vec<Route> {
    routes![test_fn, update]
}

#[post("/test", format = "application/x-www-form-urlencoded", data = "<body>")]
async fn test_fn(body: String) -> Result<String, ApiError> {
    log::info!("{}", body);
    Ok(body)
}

#[derive(Debug, Deserialize, Serialize)]
struct WebhookData {
    event: String,
    data: DealData,
    ts: i64,
    auth: Auth,
}

#[derive(Debug, Deserialize, Serialize)]
struct DealData {
    #[serde(rename = "FIELDS[ID]")]
    id: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Auth {
    domain: String,
    client_endpoint: String,
    server_endpoint: String,
    member_id: String,
    application_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Lead {
    result: LeadResult,
}

#[derive(Debug, Deserialize, Serialize)]
struct Contact {
    result: ContactResult,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct ContactResult {
    id: String,
    phone: Vec<Phone>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct LeadResult {
    id: String,
    contact_id: String,
    phone: Vec<Phone>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct Phone {
    id: String,
    value_type: String,
    value: String,
    type_id: String,
}

#[post(
    "/update",
    format = "application/x-www-form-urlencoded",
    data = "<body>"
)]
async fn update(body: String) -> Result<(), ApiError> {
    let bitrix_url = env::var("BITRIX_URL").expect("did not find BITRIX_URL env variable");
    let bitrix_token = env::var("BITRIX_TOKEN").expect("did not find BITRIX_TOKEN env variable");
    let rest_url = format!("{}/{}", bitrix_url, bitrix_token);


    // let query_string = serde_qs::from_str(&body).expect("Failed to serialize query");
    log::debug!("{}", body);

    let params: Vec<(String, String)> = form_urlencoded::parse(body.as_bytes())
        .into_owned()
        .collect();

    let mut webhook_data = WebhookData {
        event: String::new(),
        data: DealData { id: 0 },
        ts: 0,
        auth: Auth {
            domain: String::new(),
            client_endpoint: String::new(),
            server_endpoint: String::new(),
            member_id: String::new(),
            application_token: String::new(),
        },
    };

    // Iterate through the parsed key-value pairs and fill the WebhookData struct
    for (key, value) in params {
        match key.as_str() {
            "event" => webhook_data.event = value,
            "data[FIELDS][ID]" => webhook_data.data.id = value.parse().unwrap_or(0),
            "auth[domain]" => webhook_data.auth.domain = value,
            "auth[client_endpoint]" => webhook_data.auth.client_endpoint = value,
            "auth[server_endpoint]" => webhook_data.auth.server_endpoint = value,
            "auth[member_id]" => webhook_data.auth.member_id = value,
            "auth[application_token]" => webhook_data.auth.application_token = value,
            _ => (),
        }
    }

    let mut phonenumber = String::new();
    let mut phonenumbers = vec![];

    let mut contact_id = String::new();
    if webhook_data.event.contains("LEAD") {
        // curl 'https://b24-digwu0.bitrix24.de/rest/1/hqz1pc1at94get5d/crm.lead.get?id=58269' | jq
        log::info!("LEAD");

        #[derive(Serialize)]
        struct Query {
            id: i64,
        }

        // get lead
        let endpoint: String = format!("{}/crm.lead.get", rest_url);
        let query_string = serde_qs::to_string(&Query {
            id: webhook_data.data.id,
        })
        .expect("Failed to serialize query");

        let url = format!("{}?{}", &endpoint, query_string);

        let client = reqwest::Client::new();

        let response = client.get(url).send().await.unwrap();

        if response.status().is_success() {
            let body = response.text().await.unwrap();
            let lead: Result<Lead, serde_json::Error> = serde_json::from_str(&body);
            if let Ok(lead) = lead {
                log::info!("lead: {:#?}", lead);
                contact_id = lead.result.contact_id;
                log::debug!("contact: {} \n {:#?}", contact_id.clone(), phonenumbers);

                // get contact
                let endpoint: String = format!("{}/crm.contact.get", rest_url);
                let query_string: String = serde_qs::to_string(&Query {
                    id: contact_id.parse().unwrap_or(0),
                })
                .expect("Failed to serialize query");

                let url = format!("{}?{}", &endpoint, query_string);

                let client = reqwest::Client::new();

                let response = client.get(url).send().await.unwrap();

                if response.status().is_success() {
                    let body = response.text().await.unwrap();
                    let lead: Result<Contact, serde_json::Error> = serde_json::from_str(&body);
                    if let Ok(lead) = lead {
                        log::info!("contact: {:#?}", lead);
                        phonenumbers = lead.result.phone.iter().map(|p| p.value.clone()).collect();
                    } else {
                        log::error!("Error on deserialize contact: {}", body);
                    }
                } else {
                    log::error!("Error on fetch contact: {}", response.status());
                }
            } else {
                log::error!("Error on deserialize lead: {}", body);
            }
        } else {
            log::error!("Error on fetch lead: {}", response.status());
        }
    } else if webhook_data.event.contains("DEAL") {
        log::info!("DEAL");
        return Ok(());
    }

    let mut handlednumbers = vec![];

    for phonenumber in phonenumbers {
        log::debug!("номер перед обработкой: {}", phonenumber.clone());
        let phonenumber = match phonenumber
            .trim()
            .replace([' ', '-', '/', '(', ')'], "")
            .as_str()
        {
            s if s.starts_with("00") => s.replacen("00", "+", 1),
            s if s.starts_with('0') => s.replacen('0', "+49", 1),
            s if s.starts_with("490") => s.replacen("490", "+49", 1),
            s if s.starts_with("+490") => s.replacen("+490", "+49", 1),
            s if !s.starts_with('+') => format!("+{}", s),
            _ => phonenumber.to_string(),
        };
        log::debug!("номер после обработки: {}", phonenumber.clone());

        let number = phonenumber::parse(None, phonenumber);

        // error or unwrap
        if let Err(error) = number.clone() {
            log::error!("parse number error {}", error.to_string())
        }
        let number = number.unwrap();

        let valid = phonenumber::is_valid(&number);

        if valid {
            log::debug!("\x1b[32m{}\x1b[0m", "valid");
            handlednumbers.push(number);
        } else {
            log::debug!("\x1b[31m{}\x1b[0m", "invalid");
        }
    }

    let mut number_field_data = String::new();
    let mut country_field_data = String::new();
    for number in handlednumbers {
        country_field_data += format!("{:?}; ", number.country().id().unwrap()).as_str();
        number_field_data += format!("{}; ", number.format().mode(Mode::International)).as_str();
    }

    // Construct the JSON data
    let json_data = format!(

        // true for dkn:
        // UF_CRM_1690785743302 - country field in contact
        // UF_CRM_1690785780583 - phone field in contact
        r#"{{ "ID": "{}", "fields": {{ "UF_CRM_1690785743302": "{}", "UF_CRM_1690785780583": "{}" }} }}"#,
        contact_id, country_field_data, number_field_data
    );

    log::debug!("json_data: {}", json_data);

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let url = format!("{}/crm.contact.update", rest_url);
    let client = reqwest::Client::new();
    // Make the POST request
    let response = client
        .post(url)
        .headers(headers)
        .body(json_data)
        .send()
        .await
        .unwrap();

    log::debug!("good? {:#?}", response);

    Ok(())
}

#[cfg(test)]
mod test {
    use phonenumber::{country, Mode};

    #[test]
    fn check_phone() {
        let phonenumber = "+490421949720924";
        let country = country::DE;
        let number = phonenumber::parse(None, phonenumber);

        // error or unwrap
        if let Err(error) = number.clone() {
            println!("parse number error {}", error.to_string())
        }
        let number = number.unwrap();

        let valid = phonenumber::is_valid(&number);

        if valid {
            println!("\x1b[32m{}\x1b[0m", "valid");
            println!("\x1b[32m{:#?}\x1b[0m", number);
            println!(
                "international: {}",
                number.format().mode(Mode::International)
            );
            log::info!("     national: {}", number.format().mode(Mode::National));
        } else {
            println!("\x1b[31m{}\x1b[0m", "invalid");
            println!("\x1b[31m{:#?}\x1b[0m", number);
        }
    }
}
