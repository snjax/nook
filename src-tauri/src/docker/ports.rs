use bollard::Docker;

use crate::error::NookResult;
use crate::types::ExposedPort;

pub async fn get_container_port_bindings(
    docker: &Docker,
    container_id: &str,
) -> NookResult<Vec<ExposedPort>> {
    let inspect = docker
        .inspect_container(container_id, None)
        .await?;

    let mut ports = Vec::new();

    if let Some(network_settings) = inspect.network_settings {
        if let Some(port_map) = network_settings.ports {
            for (container_port_str, bindings) in port_map {
                let container_port = container_port_str
                    .split('/')
                    .next()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(0);

                if container_port == 0 {
                    continue;
                }

                if let Some(Some(binding_list)) = bindings.as_ref().map(Some) {
                    for binding in binding_list {
                        let host_port = binding
                            .host_port
                            .as_ref()
                            .and_then(|p| p.parse::<u16>().ok())
                            .unwrap_or(container_port);

                        ports.push(ExposedPort {
                            container_port,
                            host_port,
                            protocol: String::new(),
                            status: crate::types::PortStatus::Active,
                            auto_expose: false,
                        });
                    }
                }
            }
        }
    }

    Ok(ports)
}
