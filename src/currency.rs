use actix::prelude::*;
use actix::utils::IntervalFunc;
use serde::{Deserialize};
use std::collections::HashMap;
use std::time::{Duration};

const UPDATE_INTERVAL: Duration = Duration::from_secs(60);

pub struct CurrencyProvider {
  pub base: String,
  pub current_value: f32,
  pub quote: String,
}

#[derive(Deserialize, Debug)]
struct ExchangeRatesApiResult {
  base: String,
  date: String,
  rates: HashMap<String, f32>,
}

impl CurrencyProvider {
  async fn get_currency_val(base: &str, quote: &str) -> Result<f32, reqwest::Error> {
    let url = format!(
      "https://api.exchangeratesapi.io/latest?base={}&symbols={}",
      base,
      quote,
    );

    let res = reqwest::get(&url)
      .await?
      .json::<ExchangeRatesApiResult>()
      .await?;

    match res.rates.get(quote) {
      Some(v) => Ok(*v),
      None => { panic!("quote non exists into result"); },
    }
  }

  fn update_currency(&mut self, context: &mut Context<Self>) {
    let addr = context.address();
    let base = self.base.clone();
    let quote = self.quote.clone();

    Arbiter::spawn(async move {
      match Self::get_currency_val(&base, &quote).await {
        Ok(v) => {
          if let Err(e) = addr.send(UpdateValue(v)).await {
            eprintln!("Error while sending new value to CurrencyProvider. Reason: {}", e);
          }
        },
        Err(_) => { eprintln!("Failed to get currency {} -> {}", base, quote); },
      };
    });
  }
}

pub struct ActualValue;

impl Message for ActualValue {
  type Result = f32;
}

impl Handler<ActualValue> for CurrencyProvider {
  type Result = f32;

  fn handle(&mut self, _: ActualValue, _ctx: &mut Context<Self>) -> f32 {
    self.current_value
  }
}

#[derive(Debug)]
pub struct UpdateValue(f32);

impl Message for UpdateValue {
  type Result = ();
}

impl Handler<UpdateValue> for CurrencyProvider {
  type Result = ();

  fn handle(&mut self, val: UpdateValue, _ctx: &mut Context<Self>) {
    self.current_value = val.0;

    println!(
      "Update current value for {} to {} pair. Now it's {}",
      self.base,
      self.quote,
      self.current_value,
    );
  }
}

impl Actor for CurrencyProvider {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    // initial update
    Self::update_currency(self, ctx);

    IntervalFunc::new(UPDATE_INTERVAL, Self::update_currency)
      .finish()
      .spawn(ctx);
  }
}
