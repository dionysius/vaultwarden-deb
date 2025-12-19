# Diesel Logger

Let's say you use Diesel like this:
```rust
let conn = SqliteConnection::establish("example.sqlite").unwrap();
// some commands that read/write from the conn database
```
You can change this to
```rust
let conn = SqliteConnection::establish("example.sqlite").unwrap();
let conn = LoggingConnection::new(conn);
// some commands that read/write from the conn database
```
to log everything.
This produces a `debug` log on every query, an `info` on queries that take longer than 1 second, and a `warn`ing on queries that take longer than 5 seconds.

## Example

```shell
$ cd example
$ cargo run
2022-06-06T22:42:29.266Z DEBUG [diesel_logger] Query ran in 4.9 ms: CREATE TABLE IF NOT EXISTS posts (id INTEGER, title TEXT, body TEXT, published BOOL);
2022-06-06T22:42:29.270Z DEBUG [diesel_logger] Query ran in 3.7 ms: INSERT INTO `posts` (`title`, `body`) VALUES (?, ?) -- binds: ["mytitle", "mybody"]
```
