// use axum::{
//     routing::{get, post},
//     http::StatusCode,
//     response::IntoResponse,
//     Json, Router,
// };
// use serde::{Deserialize, Serialize};
// use std::net::SocketAddr;
// use std::sync::Arc;
// use axum::extract::State;
// use tokio::spawn;
//
// use tokio::sync::RwLock;
//
// pub struct Data{
//     u:u32,
// }
//
// impl Data{
//     pub fn new() -> Self {
//         Self {
//             u:0,
//         }
//     }
//     pub fn abc(& self){
//        loop{
//
//        }
//     }
// }
// #[tokio::main]
// async fn main() {
//     let data=Arc::new(RwLock::new(Data::new()));
//
//     let data1=data.clone();
//     spawn(async move{
//         data1.write().await.abc();
//     });
//
//     // build our application with a route
//     let app = Router::new()
//         // `GET /` goes to `root`
//         .route("/", get(root))
//         // `POST /users` goes to `create_user`
//         .route("/users", post(create_user))
//         .with_state(data)
//         ;
//
//     // run our app with hyper
//     // `axum::Server` is a re-export of `hyper::Server`
//     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//     axum::Server::bind(&addr)
//         .serve(app.into_make_service())
//         .await
//         .unwrap();
// }
//
// // basic handler that responds with a static string
// async fn root() -> &'static str {
//  "Hello, World!"
//
// }
//
// async fn create_user(
//     // this argument tells axum to parse the request body
//     // as JSON into a `CreateUser` type
//     Json(payload): Json<CreateUser>,
// ) -> (StatusCode, Json<User>) {
//     // insert your application logic here
//     let user = User {
//         id: 1337,
//         username: payload.username,
//     };
//
//     // this will be converted into a JSON response
//     // with a status code of `201 Created`
//     (StatusCode::CREATED, Json(user))
// }
//
// // the input to our `create_user` handler
// #[derive(Deserialize)]
// struct CreateUser {
//     username: String,
// }
//
// // the output to our `create_user` handler
// #[derive(Serialize)]
// struct User {
//     id: u64,
//     username: String,
// }



use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::task::yield_now;

async fn handle_request() -> String {
    // 执行一个长时间运行的循环，并定期使用 yield_now() 函数
    let mut i = 0;
    loop {
        i += 1;
        if i % 1000000 == 0 {
            yield_now().await;
        }
    }
    "Done".to_string()
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handle_request));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3009));

    println!("Server listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
