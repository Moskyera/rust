
#[derive(Clone)]
pub struct ServerConf {
    pub enable: bool,
    pub listen: u16,
    pub listen_host: String,
    pub allow_public_rpc: bool,
    pub multi_thread: bool,
}



impl  ServerConf {
    
    pub fn new(ini: &IniObj) -> ServerConf {
        let sec = ini_section(ini, "server");
        let mut cnf = ServerConf{
            enable:       ini_must_bool(&sec, "enable", false),
            listen:   ini_must_u64(&sec, "listen", 8083) as u16,
            listen_host: ini_must(&sec, "listen_host", "127.0.0.1"),
            allow_public_rpc: ini_must_bool(&sec, "allow_public_rpc", false),
            multi_thread: ini_must_bool(&sec, "multi_thread", false),
        };

        cnf
    }


}
