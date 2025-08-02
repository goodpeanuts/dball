use dball_client::ipc::{
    RpcService,
    client::{IpcClient, client::ClientState},
};

pub(crate) type RpcResult<T> = Result<T, String>;

static IPC_CLIENT: async_lazy::Lazy<IpcClient> = async_lazy::Lazy::new(|| {
    Box::pin(async {
        IpcClient::new_connected()
            .await
            .expect("Failed to connect to IPC client")
    })
});

#[expect(unused)]
pub async fn get_ipc_client_state() -> ClientState {
    let client = IPC_CLIENT.force().await;
    client.get_state().await
}

pub async fn send_rpc_request<T>(service: RpcService) -> RpcResult<T>
where
    for<'de> T: serde::Deserialize<'de>,
{
    let client = IPC_CLIENT.force().await;
    match client.send_rpc_request(service).await {
        Ok(response) => serde_json::from_value::<T>(response).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}
