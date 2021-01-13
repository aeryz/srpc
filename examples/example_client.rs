use {
    srpc::{client::Client, transport::Transport},
    std::sync::Arc,
};

#[srpc::client]
trait StrService {
    async fn contains(data: String, elem: String) -> bool;

    #[notification]
    async fn set_data(is_cool: bool);
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let transporter = Arc::new(Transport::new());
    let client = Client::new(([127, 0, 0, 1], 8080).into(), transporter.clone());

    for i in 0..100 {
        let _ = StrService::set_data(&client, i % 2 == 0).await;
        println!(
            "{}",
            StrService::contains(&client, String::from("cool lib"), String::from("lib"))
                .await
                .unwrap()
        );
    }
}
