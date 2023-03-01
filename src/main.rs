use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
pub struct ComicInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    writers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    artists: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    release_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cover_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    critic_review_count: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_review_count: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    critic_review_score: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_review_score: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ComicInfos {
    #[serde(skip_serializing_if = "Option::is_none")]
    comic_infos: Option<Vec<ComicInfo>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LambdaResponse {
    pub urls: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublisherResponse {
    pub comics: Vec<Comic>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Comic {
    pub name: String,
    pub url: String,
}

async fn make_request(
    client: &reqwest::Client,
    sem: Arc<Semaphore>,
    url: &str,
) -> Result<String, reqwest::Error> {
    let get_comic_issue_details_url = "http://localhost:8080/details";
    let body = json!({ "url": url });
    let _permit = sem.acquire().await.unwrap();
    let response = client
        .post(get_comic_issue_details_url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await?
        .text()
        .await?;
    Ok(response)
}

#[tokio::main]
async fn main() {
    let list_publisher_comics = "http://localhost:8080/comics";
    let body = json!({
        "name": "valiant"
    });
    let list_comic_issues_url = "http://localhost:8080/issues";

    let response = reqwest::Client::new()
        .post(list_publisher_comics)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let response: PublisherResponse = serde_json::from_str(&response.replace('\\', "")).unwrap();

    let sem = Arc::new(Semaphore::new(5000)); // Allow 4 requests at a time
    let client = Client::new();

    let vecs: Arc<Mutex<Vec<ComicInfo>>> = Arc::new(Mutex::new(Vec::new()));

    for comic in response.comics {
        let json_body = json!({ "url": comic.url });
        {
            let result = {
                let response = reqwest::Client::new()
                    .post(list_comic_issues_url)
                    .header("Content-Type", "application/json")
                    .body(json_body.to_string())
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap();
                response
            };
            let response: LambdaResponse =
                serde_json::from_str(&result).unwrap();

            let client = client.clone();
            let sem = Arc::clone(&sem);

            for url in response.urls {
                let client = client.clone();
                let sem = Arc::clone(&sem);
                let vecs_clone = Arc::clone(&vecs);
                tokio::spawn(async move {
                    let result = make_request(&client, sem, &url).await;
                    match result {
                        Ok(body) => {
                            let mut vecs = vecs_clone.lock().unwrap();
                            let comic_info: ComicInfo =
                                serde_json::from_str(&body).unwrap();
                                vecs.push(comic_info);
                        },
                        Err(e) => eprintln!("Request error: {}", e),
                    }
                });
            }
        };
    }

    // Wait for all tasks to complete before exiting the program
    tokio::task::yield_now().await;
    let mutex_as_vec = vecs.lock().unwrap();
    mutex_as_vec.iter().for_each(
        |x| println!("{:?}", x)
    );
}



// use reqwest::Client;
// use serde::{Deserialize, Serialize};
// use serde_json::json;

// #[derive(Serialize, Deserialize, Debug)]
// pub struct LambdaResponse {
//     pub urls: Vec<String>,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct PublisherResponse {
//     pub comics: Vec<Comic>,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Comic {
//     pub name: String,
//     pub url: String,
// }

// async fn make_request(client: &reqwest::Client, url: &str) -> Result<String, reqwest::Error> {
//     let get_comic_issue_details_url = "http://localhost:8080/details";
//     let body = json!({ "url": url });
//     let response = client
//         .post(get_comic_issue_details_url)
//         .header("Content-Type", "application/json")
//         .body(body.to_string())
//         .send()
//         .await?
//         .text()
//         .await?;
//     Ok(response)
// }

// #[tokio::main]
// async fn main() {
//     let list_publisher_comics = "http://localhost:8080/comics";
//     let body = json!({
//         "name": "dc"
//     });
//     let list_comic_issues_url = "http://localhost:8080/issues";

//     let response = reqwest::Client::new()
//         .post(list_publisher_comics)
//         .header("Content-Type", "application/json")
//         .body(body.to_string())
//         .send()
//         .await
//         .unwrap()
//         .text()
//         .await
//         .unwrap();

//     let response: PublisherResponse = serde_json::from_str(&response.replace('\\', "")).unwrap();

//     for comic in response.comics {
//         let json_body = json!({ "url": comic.url });
//         {
//             let result = {
//                 let response = reqwest::Client::new()
//                     .post(list_comic_issues_url)
//                     .header("Content-Type", "application/json")
//                     .body(json_body.to_string())
//                     .send()
//                     .await
//                     .unwrap()
//                     .text()
//                     .await
//                     .unwrap();
//                 response
//             };
//             let response: LambdaResponse =
//                 serde_json::from_str(&result).unwrap();
//             let client = Client::new();

//             for url in response.urls {
//                 let client = client.clone();
//                 {
//                     let result = make_request(&client, &url).await;
//                     match result {
//                         Ok(body) => println!("{}", body),
//                         Err(e) => eprintln!("Request error: {}", e),
//                     }
//                 };
//             }
//         };
//     }
// }

