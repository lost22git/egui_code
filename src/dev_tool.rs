use std::process::Command;

use crate::util;

///////////////////////////////////////////////
// puffin_server
///////////////////////////////////////////////
const SERVER_ADDR: &str = "127.0.0.1:8585";

pub fn start_puffin_server() {
  puffin::set_scopes_on(true,);

  match puffin_http::Server::new(SERVER_ADDR,) {
    Ok(puffin_server,) => {
      // We can store the server if we want, but in this case we just want
      // it to keep running. Dropping it closes the server, so let's not drop it!
      #[allow(clippy::mem_forget)]
      std::mem::forget(puffin_server,);
    }
    Err(err,) => {
      util::toaster()
        .error(format!(
          "启动 puffin server 失败，server_addr={}，错误：{:?}",
          SERVER_ADDR, err
        ),)
        .set_duration(Some(std::time::Duration::from_secs(5,),),);
    }
  };
}

pub fn open_puffin_viewer() {
  let result = Command::new("puffin_viewer",)
    .args(["--url", SERVER_ADDR,],)
    .spawn();

  match result {
    Ok(_,) => {}
    Err(_,) => {
      util::toaster()
                .error(
                    "启动 puffin_viewer 失败，请检查是否已安装 puffin viewer，尝试执行 `cargo install puffin_viewer`",
                )
                .set_duration(Some(std::time::Duration::from_secs(5,),),);
    }
  }
}
