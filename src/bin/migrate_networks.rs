use std::env;

use dotenv::dotenv;
use log::LevelFilter;
use vivalaakam_neat_rs::{Connection, Genome, NeuronType, Node, Organism};

use experiments::{load_networks, on_add_network, save_parse_network, Parse};

#[tokio::main]
async fn main() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .is_test(true)
        .try_init();

    dotenv().ok();

    let parse = Parse::new(
        env::var("PARSE_REMOTE_URL").expect("PARSE_REMOTE_URL must be set"),
        env::var("PARSE_APP_ID").expect("PARSE_APP_ID must be set"),
        env::var("PARSE_REST_KEY").expect("PARSE_REST_KEY must be set"),
    );

    let networks = load_networks(&parse, 180, 2).await;

    for network in networks {
        let start_node = "output_1".to_string();
        let end_node = "output_4".to_string();

        let nodes_to_add = vec![
            "output_1".to_string(),
            "output_2".to_string(),
            "output_3".to_string(),
        ];

        let genome: Genome = network.network.into();

        let mut nodes = vec![];

        for node in genome.get_nodes() {
            let node_id = if node.get_id() == start_node {
                end_node.to_string()
            } else {
                node.get_id()
            };

            let position = if node.get_id() == start_node {
                node.get_position() + 4
            } else {
                node.get_position()
            };

            let mut n = Node::new(
                node.get_type(),
                node_id,
                node.get_bias(),
                Some(node.get_activation()),
                Some(position),
            );

            if node.get_enabled() == false {
                n.toggle_enabled();
            }

            if node.get_id() == start_node {
                for i in 0..nodes_to_add.len() {
                    let id = nodes_to_add[i].to_string();
                    nodes.push(Node::new(
                        node.get_type(),
                        id.to_string(),
                        0f64,
                        Some(node.get_activation()),
                        Some(node.get_position() + i),
                    ));
                }
            }

            nodes.push(n);
        }

        let mut connections = vec![];

        for connection in genome.get_connections() {
            let mut conn = Connection::new(
                connection.get_from(),
                if connection.get_to() == start_node {
                    end_node.to_string()
                } else {
                    connection.get_to()
                },
                connection.get_weight(),
            );

            conn.set_enabled(connection.get_enabled());

            connections.push(conn);
        }

        let inputs = genome
            .get_nodes()
            .into_iter()
            .filter(|n| n.get_type() == NeuronType::Input)
            .map(|n| n.get_id())
            .collect::<Vec<_>>();

        for input in &inputs {
            for output in &nodes_to_add {
                connections.push(Connection::new(input.to_string(), output.to_string(), 0f64))
            }
        }

        let organism = Organism::new(Genome::new(nodes, connections));

        let network_id = save_parse_network(&parse, &organism, 180, 5).await;

        on_add_network(&parse, network_id).await;
    }
}
