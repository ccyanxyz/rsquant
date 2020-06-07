
pub struct WsClient {
    out: Sender,
    connected_callback: Box<dyn FnMut()>,
    process_binary: Box<dyn FnMut()>,
}

impl WsClient {
    pub fn new() -> Self {

    }
}

impl Handler for WsClient {

}
