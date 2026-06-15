

impl RPCServer {

    pub fn start(mut self) {
        if !self.cnf.enable {
            return // disable
        }
        let rt = new_tokio_rt(self.cnf.multi_thread);
        // server listen loop
        rt.block_on(async move {
            server_listen(self).await
        });
    }

}


async fn server_listen(mut ser: RPCServer) {
    use axum::extract::DefaultBodyLimit;
    use axum::Extension;
    use std::net::IpAddr;
    use crate::server::security::{
        MiddlewareCtx, RPC_BODY_LIMIT_BYTES, resolve_listen_endpoint, security_middleware,
    };

    let port = ser.cnf.listen;
    let host = match resolve_listen_endpoint(&ser.cnf.listen_host, port, ser.cnf.allow_public_rpc) {
        Ok(h) => h,
        Err(e) => {
            println!("\n[Error] RPC Server config: {}\n", e);
            return;
        }
    };
    let ip: IpAddr = host.parse().unwrap_or_else(|_| "127.0.0.1".parse().unwrap());
    let addr = SocketAddr::from((ip, port));
    let listener = TcpListener::bind(addr).await;
    if let Err(ref e) = listener {
        println!("\n[Error] RPC Server bind {}:{} error: {}\n", host, port, e);
        return
    }
    let listener = listener.unwrap();
    println!("[RPC Server] Listening on http://{addr}");
    if crate::server::security::is_public_bind_host(&host) {
        println!("[RPC Server] WARNING: public bind enabled (allow_public_rpc=true)");
    }
    //
    let mw = MiddlewareCtx {
        listen_host: host.clone(),
        listen_port: port,
        rate_limiter: Arc::new(crate::server::security::RateLimiter::new(60, 60)),
    };
    let ctx = ApiCtx::new(
        ser.engine.clone(),
        ser.hcshnd.clone(),
        host,
    );
    let app = rpc::routes(ctx)
        .layer(DefaultBodyLimit::max(RPC_BODY_LIMIT_BYTES))
        .layer(Extension(mw))
        .layer(axum::middleware::from_fn(security_middleware));
    println!("[RPC Server] HIP-25 wallet UI: http://{addr}/hip25/wallet");
    let make_svc = app.into_make_service_with_connect_info::<SocketAddr>();
    if let Err(e) = axum::serve(listener, make_svc).await {
        println!("{e}");
    }
}