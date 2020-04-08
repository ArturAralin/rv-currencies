use std::collections::HashMap;
use actix_web::{HttpServer, App, HttpRequest, HttpResponse, middleware, web, dev};
use actix::{Addr};
use serde_json::json;
use qstring::QString;
use crate::currency::{CurrencyProvider, CurrentValue};

fn ok_json_response(v: serde_json::Value) -> web::HttpResponse {
  HttpResponse::Ok()
    .header("Content-Type", "application/json")
    .body(v)
}

async fn get_currency(
  currency_providers: web::Data<HashMap<String, Addr<CurrencyProvider>>>,
  req: HttpRequest
) -> HttpResponse {
  let query = QString::from(req.query_string());
  let pair = query.get("pair");

  if let None = pair {
    return ok_json_response(json!({
      "status": "pair_not_provided"
    }))
  }

  if let Some(provider) = currency_providers.get("USD_RUB") {
    return match provider.send(CurrentValue).await {
      Ok(v) => {
        ok_json_response(json!({
          "status": "ok",
          "current_value": v
        }))
      },
      Err(e) => {
        HttpResponse::InternalServerError()
          .header("Content-Type", "application/json")
          .body(json!({
            "status": "internal_error",
            "reason": format!("{}", e),
          }))
      }
    };
  }

  ok_json_response(json!({
    "status": "pair_not_found",
  }))
}

pub fn start_server(
  currency_providers: HashMap<String, Addr<CurrencyProvider>>
) -> Result<dev::Server, std::io::Error> {
  std::env::set_var("RUST_LOG", "actix_web=info");
  env_logger::init();

  let srv = HttpServer::new(move || {
    App::new()
      .data(currency_providers.clone())
      .wrap(middleware::Logger::default())
      .service(web::resource("/get_currency").to(get_currency))
  })
  .bind("localhost:8888")?
  .run();

  Ok(srv)
}
