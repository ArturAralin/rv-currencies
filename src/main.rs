mod currency;
mod rest;

use std::fs::File;
use std::io::prelude::*;
use actix::prelude::*;
use currency::{CurrencyProvider};
use std::collections::HashMap;

struct Pair {
  base: String,
  quote: String,
}

fn load_pairs() -> Result<Vec<Pair>, std::io::Error> {
  let mut file = File::open("./resources/pairs.txt")?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;

  let pairs = contents
    .split("\n")
    .filter(|s| s.contains("->"))
    .map(|s| {
      let v = s
        .split("->")
        .map(|s| s.trim())
        .collect::<Vec<&str>>();

      Pair {
        base: String::from(v[0]),
        quote: String::from(v[1]),
      }
    })
    .collect::<Vec<Pair>>();

  Ok(pairs)
}

fn main() -> Result<(), std::io::Error> {
  let system = System::new("rt");

  let currency_providers = match load_pairs() {
    Ok(v) => {
      v.iter().map(move |pair| {
        let base = pair.base.clone();
        let quote = pair.quote.clone();
        let pair_id = format!("{}_{}", base, quote);

        let addr = CurrencyProvider::create(move |_| {
          CurrencyProvider {
            base: base,
            quote: quote,
            current_value: 0.0,
          }
        });

        (pair_id, addr)
      }).collect::<HashMap<String, actix::Addr<CurrencyProvider>>>()
    },
    Err(e) => { panic!("Error while loading pairs {}", e); }
  };

  match rest::start_server(currency_providers) {
    Ok(_) => {
      println!("Web server successfully started");
    },
    Err(e) => {
      panic!("Error while starting web server: {}", e);
    }
  };

  system.run()
}
