## Extract

Trait implemented by types that can be extracted from [Context](context.md).

```rust
#[derive(Debug)]
struct Info {
    path: String,
    method: String,
}

impl Extract for Info {
    type Error = Error;

    fn extract(cx: &mut Context) -> BoxFuture<'_, Result<Self, Self::Error>> {
        Box::pin(async move {
            let method = cx.method().to_string();
            // if method is PUT, throw an error
            if method == "PUT" {
                return Err(anyhow!("Wrong HTTP Method!"));
            }
            Ok(Info {
                method,
                path: cx.path().to_string(),
            })
        })
    }
}

async fn handler(info: Info) -> String {
    format!("{:#?}", info)
}
```

### Built-in Extractors

* `Cookies`

* `Form<T>`

* `Header<T>`

* `Json<T>`

* `Multipart<T>`

* `Params<T>`

* `Query<T>`

* `State<T>`
