use std::collections::HashMap;

/// Returns the protocol for a well-known port number.
pub fn protocol_for_port(port: u16, user_overrides: &HashMap<String, String>) -> Option<String> {
    if let Some(proto) = user_overrides.get(&port.to_string()) {
        return Some(proto.clone());
    }

    let proto = match port {
        80 | 8080 | 8000 | 8888 | 3000 | 3001 | 4200 | 5000 | 5173 | 5174 | 5500 | 9000 => {
            "http"
        }
        443 | 8443 => "https",
        5432 => "postgres",
        3306 | 33060 => "mysql",
        6379 => "redis",
        27017 | 27018 => "mongodb",
        5672 | 15672 => "amqp",
        9200 | 9300 => "elasticsearch",
        8500 | 8501 => "consul",
        22 => "ssh",
        25 | 587 | 465 => "smtp",
        53 => "dns",
        6443 => "kubernetes",
        2181 => "zookeeper",
        9092 => "kafka",
        11211 => "memcached",
        1433 => "mssql",
        1521 => "oracle",
        _ => return None,
    };

    Some(proto.to_string())
}

/// Returns the likely protocol based on the process name.
pub fn protocol_for_process(process_name: &str) -> Option<String> {
    let name = process_name
        .rsplit('/')
        .next()
        .unwrap_or(process_name)
        .to_lowercase();

    let proto = match name.as_str() {
        "node" | "deno" | "bun" | "python" | "python3" | "ruby" | "php" | "java" | "go"
        | "dotnet" | "uvicorn" | "gunicorn" | "flask" | "django" | "next-server"
        | "webpack" | "vite" | "nginx" | "apache2" | "httpd" | "caddy" => "http",
        "postgres" | "pg_isready" | "postgresql" => "postgres",
        "mysqld" | "mariadbd" | "mysql" => "mysql",
        "redis-server" | "redis-sentinel" => "redis",
        "mongod" | "mongos" => "mongodb",
        "rabbitmq-server" | "beam.smp" => "amqp",
        "elasticsearch" | "opensearch" => "elasticsearch",
        "sshd" => "ssh",
        _ => return None,
    };

    Some(proto.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_well_known_ports() {
        let empty: HashMap<String, String> = HashMap::new();
        assert_eq!(protocol_for_port(80, &empty), Some("http".to_string()));
        assert_eq!(protocol_for_port(443, &empty), Some("https".to_string()));
        assert_eq!(protocol_for_port(5432, &empty), Some("postgres".to_string()));
        assert_eq!(protocol_for_port(3306, &empty), Some("mysql".to_string()));
        assert_eq!(protocol_for_port(6379, &empty), Some("redis".to_string()));
        assert_eq!(protocol_for_port(27017, &empty), Some("mongodb".to_string()));
        assert_eq!(protocol_for_port(22, &empty), Some("ssh".to_string()));
        assert_eq!(protocol_for_port(3000, &empty), Some("http".to_string()));
        assert_eq!(protocol_for_port(5173, &empty), Some("http".to_string()));
        assert_eq!(protocol_for_port(8080, &empty), Some("http".to_string()));
    }

    #[test]
    fn test_unknown_ports() {
        let empty: HashMap<String, String> = HashMap::new();
        assert_eq!(protocol_for_port(12345, &empty), None);
        assert_eq!(protocol_for_port(55555, &empty), None);
    }

    #[test]
    fn test_user_overrides() {
        let overrides = HashMap::from([("9999".to_string(), "custom-proto".to_string())]);
        assert_eq!(
            protocol_for_port(9999, &overrides),
            Some("custom-proto".to_string())
        );
        // User override takes precedence
        let overrides = HashMap::from([("80".to_string(), "custom-http".to_string())]);
        assert_eq!(
            protocol_for_port(80, &overrides),
            Some("custom-http".to_string())
        );
    }

    #[test]
    fn test_process_names() {
        assert_eq!(protocol_for_process("node"), Some("http".to_string()));
        assert_eq!(protocol_for_process("python3"), Some("http".to_string()));
        assert_eq!(
            protocol_for_process("redis-server"),
            Some("redis".to_string())
        );
        assert_eq!(protocol_for_process("postgres"), Some("postgres".to_string()));
        assert_eq!(protocol_for_process("mysqld"), Some("mysql".to_string()));
        assert_eq!(protocol_for_process("mongod"), Some("mongodb".to_string()));
    }

    #[test]
    fn test_process_with_path() {
        assert_eq!(
            protocol_for_process("/usr/bin/node"),
            Some("http".to_string())
        );
        assert_eq!(
            protocol_for_process("/usr/lib/postgresql/14/bin/postgres"),
            Some("postgres".to_string())
        );
    }

    #[test]
    fn test_unknown_process() {
        assert_eq!(protocol_for_process("my-custom-app"), None);
        assert_eq!(protocol_for_process("unknown-daemon"), None);
    }
}
