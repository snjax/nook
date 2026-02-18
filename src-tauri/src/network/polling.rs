use regex::Regex;

/// Parsed listening port from ss output
#[derive(Debug, Clone, PartialEq)]
pub struct ListeningPort {
    pub port: u16,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
}

/// Parse output of `ss -tlnp` to extract listening ports
pub fn parse_ss_output(output: &str) -> Vec<ListeningPort> {
    let re_addr = Regex::new(r"(?:[\d.*]+|[\[:*\]]+):(\d+)").unwrap();
    let re_proc = Regex::new(r#"users:\(\("([^"]+)",pid=(\d+)"#).unwrap();

    let mut ports = Vec::new();

    for line in output.lines().skip(1) {
        // Skip header
        if !line.starts_with("LISTEN") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let local_addr = parts[3];
        let port = match re_addr.captures(local_addr) {
            Some(caps) => caps[1].parse::<u16>().unwrap_or(0),
            None => continue,
        };

        if port == 0 {
            continue;
        }

        let (pid, process_name) = if let Some(caps) = re_proc.captures(line) {
            let name = caps[1].to_string();
            let pid = caps[2].parse::<u32>().ok();
            (pid, Some(name))
        } else {
            (None, None)
        };

        // Avoid duplicates (IPv4 and IPv6 on same port)
        if !ports.iter().any(|p: &ListeningPort| p.port == port) {
            ports.push(ListeningPort {
                port,
                pid,
                process_name,
            });
        }
    }

    ports
}

/// Parse /proc/net/tcp hex format to extract listening ports
pub fn parse_proc_net_tcp(output: &str) -> Vec<ListeningPort> {
    let mut ports = Vec::new();

    for line in output.lines().skip(1) {
        // Skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        // State 0A = LISTEN
        let state = parts[3];
        if state != "0A" {
            continue;
        }

        // local_address format: hex_ip:hex_port
        let local_addr = parts[1];
        if let Some(hex_port) = local_addr.split(':').nth(1) {
            if let Ok(port) = u16::from_str_radix(hex_port, 16) {
                if port > 0 && !ports.iter().any(|p: &ListeningPort| p.port == port) {
                    ports.push(ListeningPort {
                        port,
                        pid: None,
                        process_name: None,
                    });
                }
            }
        }
    }

    ports
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ss_output() {
        let output = r#"State  Recv-Q Send-Q Local Address:Port  Peer Address:Port Process
LISTEN 0      511          0.0.0.0:3000       0.0.0.0:*     users:(("node",pid=1234,fd=3))
LISTEN 0      128          0.0.0.0:5432       0.0.0.0:*     users:(("postgres",pid=5678,fd=5))
LISTEN 0      511             [::]:3000          [::]:*     users:(("node",pid=1234,fd=4))
"#;
        let ports = parse_ss_output(output);
        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0].port, 3000);
        assert_eq!(ports[0].process_name, Some("node".to_string()));
        assert_eq!(ports[0].pid, Some(1234));
        assert_eq!(ports[1].port, 5432);
        assert_eq!(ports[1].process_name, Some("postgres".to_string()));
    }

    #[test]
    fn test_parse_ss_output_no_process() {
        let output = r#"State  Recv-Q Send-Q Local Address:Port  Peer Address:Port Process
LISTEN 0      128    *:8080                *:*
"#;
        let ports = parse_ss_output(output);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].port, 8080);
        assert_eq!(ports[0].process_name, None);
    }

    #[test]
    fn test_parse_proc_net_tcp() {
        let output = r#"  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 00000000:0BB8 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 12345
   1: 00000000:1538 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 12346
   2: 0100007F:0035 00000000:0000 01 00000000:00000000 00:00000000 00000000     0        0 12347
"#;
        let ports = parse_proc_net_tcp(output);
        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0].port, 3000); // 0x0BB8
        assert_eq!(ports[1].port, 5432); // 0x1538
    }

    #[test]
    fn test_parse_empty_output() {
        assert!(parse_ss_output("").is_empty());
        assert!(parse_proc_net_tcp("").is_empty());
    }
}
