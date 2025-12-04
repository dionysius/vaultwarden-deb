#[macro_use] extern crate rocket;

struct Q;

#[get("/<_foo>")]
fn f0(_foo: Q) {}

#[get("/<_foo..>")]
fn f1(_foo: Q) {}

#[get("/?<_foo>")]
fn f2(_foo: Q) {}

#[get("/?<_foo..>")]
fn f3(_foo: Q) {}

#[post("/", data = "<_foo>")]
fn f4(_foo: Q) {}

#[get("/<_foo>")]
fn f5(_a: Q, _foo: Q) {}

#[get("/<_foo>/other/<_bar>/<_good>/okay")]
fn f6(_a: Q, _foo: Q, _good: usize, _bar: Q) {}

fn main() {  }
