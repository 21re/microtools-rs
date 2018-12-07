use super::retry::Retrier;
use super::{business_result, Problem};
use actix::{Arbiter, System};
use futures::Future;
use spectral::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

#[test]
fn test_retry_ok() {
  let system = System::new("test");
  let retry: Retrier = Default::default();
  let tries = Arc::new(Mutex::new(0));

  Arbiter::spawn(
    retry
      .retry(tries.clone(), |count_tries| {
        let mut t = count_tries.lock().unwrap();
        *t += 1;
        business_result::success(42)
      })
      .then(move |result| {
        assert_that(&result).is_equal_to(Ok(42));
        System::current().stop();
        Ok(())
      }),
  );

  let exit = system.run();

  assert_that(&exit).is_equal_to(0);
  let t = *tries.lock().unwrap();
  assert_that(&t).is_equal_to(1);
}

#[test]
fn test_retry_all_fail() {
  let system = System::new("test");
  let retry: Retrier = Retrier::new(5, Duration::from_millis(100));
  let tries = Arc::new(Mutex::new(0));
  let times = Arc::new(Mutex::new(Vec::<SystemTime>::new()));

  Arbiter::spawn(
    retry
      .retry((tries.clone(), times.clone()), |(count_tries, times_collect)| {
        let mut t = count_tries.lock().unwrap();
        *t += 1;
        let mut times = times_collect.lock().unwrap();
        times.push(SystemTime::now());
        business_result::failure::<u32, _>(Problem::not_found().with_details("Always fail"))
      })
      .then(move |result| {
        assert_that(&result).is_equal_to(Err(Problem::not_found().with_details("Always fail")));
        System::current().stop();
        Ok(())
      }),
  );

  let exit = system.run();

  assert_that(&exit).is_equal_to(0);
  let t = *tries.lock().unwrap();
  assert_that(&t).is_equal_to(5);
  let call_times: &Vec<SystemTime> = &times.lock().unwrap();
  assert_that(call_times).has_length(5);
  let diffs: Vec<Duration> = call_times
    .iter()
    .zip(call_times.iter().skip(1))
    .map(|(a, b)| b.duration_since(*a).unwrap())
    .collect();
  assert_that(&diffs).has_length(4);
  assert!(diffs.iter().all(|d| *d > Duration::from_millis(90)));
}

#[test]
fn test_retry_nfail() {
  let system = System::new("test");
  let retry: Retrier = Retrier::new(5, Duration::from_millis(500));
  let tries = Arc::new(Mutex::new(0));
  let times = Arc::new(Mutex::new(Vec::<SystemTime>::new()));

  Arbiter::spawn(
    retry
      .retry((tries.clone(), times.clone()), |(count_tries, times_collect)| {
        let mut t = count_tries.lock().unwrap();
        *t += 1;
        let mut times = times_collect.lock().unwrap();
        times.push(SystemTime::now());
        if *t > 2 {
          business_result::success(42)
        } else {
          business_result::failure::<u32, _>(Problem::not_found().with_details("Fail"))
        }
      })
      .then(move |result| {
        assert_that(&result).is_equal_to(Ok(42));
        System::current().stop();
        Ok(())
      }),
  );

  let exit = system.run();

  assert_that(&exit).is_equal_to(0);
  let t = *tries.lock().unwrap();
  assert_that(&t).is_equal_to(3);
  let call_times: &Vec<SystemTime> = &times.lock().unwrap();
  assert_that(call_times).has_length(3);
  let diffs: Vec<Duration> = call_times
    .iter()
    .zip(call_times.iter().skip(1))
    .map(|(a, b)| b.duration_since(*a).unwrap())
    .collect();
  assert_that(&diffs).has_length(2);
  assert!(diffs.iter().all(|d| *d > Duration::from_millis(490)));
}
