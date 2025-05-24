use std::net::SocketAddr;

use crate::cmd::{CommandKind, Error, ResponseKind};

use super::registry::CommandRegistryHandle;

////////////////////////////////////////////////////////////////////////////////

async fn serve_get(
    reg: CommandRegistryHandle,
    axum::Json(cmd): axum::Json<CommandKind>,
) -> axum::Json<Result<ResponseKind, Error>> {
    println!("Handling command: {cmd:?}");
    let token = reg.register(cmd).await;
    let result = token.await.unwrap();
    axum::Json(result)
}

////////////////////////////////////////////////////////////////////////////////

// Get CommandKind from user
pub async fn serve(addr: SocketAddr, reg: CommandRegistryHandle) {
    let app = axum::Router::new().route("/get", axum::routing::get(|cmd| serve_get(reg, cmd)));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::cmd::CommandKind;

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn works() {
        let cmd = CommandKind::Read { key: "k".into() };
        // let cmd = axum::Json(cmd);
        // println!("{cmd:?}");
        let cmd = serde_json::to_string(&cmd).unwrap();
        println!("{cmd}");
    }
}
