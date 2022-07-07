use lambda::{handler_fn, Context};
use serde_json::Value;
use chrono::Utc;
use lambda_runtime::{error::HandlerError, lambda, Context};
use log::debug;
use rand::thread_rng;
use rand::seq::IteratorRandom;
use rusoto_core::Region;
use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient, PutItemError, PutItemInput, PutItemOutput};
use serde_derive::{Serialize, Deserialize};
use std::collections::HashMap;
use std::error::Error;
use uuid::Uuid

// A main function that exports the handler of a lambda function 
// And initializes simple_logger to read with CloudWatch service
fm() -> Result<(), Box<dyn Error>
{
    simple_logger::init_with_level(log::Level::Debug)?;   
    debug!("Rustacean! Executing lambda!");
    lambda!(handler);
    Ok(())
}


// A handler function that expects a request and returns a response
fn handler(event: Request, _: Context) -> Result<Response, HandlerError>
{
    /* Create a connection to DynamoDB using default Region value,
    which reads environment variable to get actual value, 
    if it does not find region, it uses us-east-1 region */
    let region = Region::default();        
    let client = DynamoDbClient::new(region);

    let username = event
        .request_context
        .authorize
        .claims
        // Extract username provided by Cognito to authorize users, no need for manual user registration
        .get("cognito:username")
        .unwrap()
        .to_owned();
    debug!("USERNAME: {}", username);

    // Generate unique ID of a ride 
    let ride_id = Uuid::new_v4().to_string();
    
    // Extract the body of a request that is provided by a JSON string
    let request: RequestBody = serde_json::from_str(&event.body).unwrap();
    let taxi = find_taxi(&request.pickup.location);

    // Add a record to a database
    record_ride(&client, &ride_id, &username, &taxi).unwrap();

    // Create body of response when record is added
    let body = ResponseBody
    {
        ride_id: ride_id.clone();
        taxi_name: taxi.name.close();
        taxi,
        eta: "30 seconds".into(),
        passenger: username.clone(),
    };

    // Use API Gateway to call the lambda with an external application shared by S3
    let mut headers = HashMap::new();
    headers.insert("Access-Control-Allow-Origin".into(), "*".into());
    let body = serde_json::to_string(&body).unwrap();
    let resp = Response
    {
        status_code: 201,
        body,
        headers,
    };

    Ok(resp)
        
}

// Main struct Taxi, contains the car passenger rides
#[derive(Clone, Serialize)]
 #[serde(rename_all = "PascalCase")]
 struct Taxi
 {
    name: String,
    color: String,
    manufacturer: String,
    
 }

 // Constructor of Taxi, we can use enumeration for these values but we need to be sure the serialized is what we want exactly
 impl Taxi
 {
        fn new(name: &str, color: &str, manufacturer: &str) -> Self
        {
            Taxi
            {
                name: name.to_owned(),
                color: color.to_owned(),
                manufacturer: manufacturer.to_owned(),
            }  
        }
 }


//  Location struct that represents a point on a map that will be set by the UI of the application
#[derive(Deserialize)]
 #[serde(rename_all = "PascalCase")]
 struct Location
 {
    latitude: f64,
    longtitude: f64,
 }