use super::*;
use crate::search_backend::transport::{SearchDns, TransportBudget, tests::Server};
use async_trait::async_trait;
use std::{
    net::SocketAddr,
    sync::{Arc, atomic::{AtomicUsize, Ordering}},
    time::Duration,
};
use talos_core::tool::{
    ToolNetworkAuthority, ToolNetworkAuthorizationError, ToolNetworkEndpoint, ToolNetworkManifest,
    ToolNetworkScope,
};

struct Effects {
    manifest: ToolNetworkManifest,
    dns: AtomicUsize,
}

impl ToolNetworkAuthority for Effects {
    fn check(
        &self,
        invocation: uuid::Uuid,
        endpoint: &ToolNetworkEndpoint,
    ) -> Result<(), ToolNetworkAuthorizationError> {
        if invocation == self.manifest.invocation_id()
            && self.manifest.scope().contains(endpoint)
        {
            Ok(())
        } else {
            Err(ToolNetworkAuthorizationError::Mismatch)
        }
    }
}

#[async_trait]
impl SearchDns for Effects {
    async fn resolve(&self, _: &str, port: u16) -> Result<Vec<SocketAddr>, TransportError> {
        self.dns.fetch_add(1, Ordering::Relaxed);
        Ok(vec![SocketAddr::from(([127, 0, 0, 1], port))])
    }
}

fn fixed_manifest() -> ToolNetworkManifest {
    ToolNetworkManifest::new(
        ToolNetworkScope::new_configuration(
            ORDER
                .iter()
                .map(|protocol| protocol.destination().expect("destination"))
                .collect(),
            None,
            vec![],
            0,
        )
        .expect("scope"),
    )
}

fn response(body: &[u8]) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        String::from_utf8(body.to_vec()).expect("fixture UTF-8")
    )
}

#[tokio::test]
async fn unreachable_duckduckgo_family_opens_only_the_reachable_bing_query_source() {
    let server = Server::new(vec![response(&super::super::execution_tests::body(
        CompatibilityProtocol::Bing,
    ))]);
    let chain = PreparedCompatibilityChain::prepare(&fixed_manifest(), "private + query", 1)
        .expect("prepared chain");
    let (chain, manifest) = super::tests::relocate(chain, &server);
    let effects = Arc::new(Effects {
        manifest: manifest.clone(),
        dns: AtomicUsize::new(0),
    });
    let transport = SearchTransport::new(
        manifest,
        effects.clone(),
        Arc::new(
            TransportBudget::new(Duration::from_secs(5), 2, 1_048_576).expect("budget"),
        ),
        effects.clone(),
    )
    .expect("transport");

    let rows = chain
        .execute_with_reachability(
            &transport,
            Reachability {
                duckduckgo: false,
                bing: true,
            },
        )
        .await
        .expect("reachable Bing fallback")
        .into_results();

    assert_eq!(rows.len(), 1);
    assert_eq!(server.count(), 1);
    let sent = server.request(0);
    assert!(sent.starts_with("GET /search?"));
    assert!(sent.contains("q=private+%2B+query"));
    assert_eq!(effects.dns.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn both_unreachable_families_send_no_query_bearing_request() {
    let server = Server::new(vec![]);
    let chain = PreparedCompatibilityChain::prepare(&fixed_manifest(), "must stay local", 1)
        .expect("prepared chain");
    let (chain, manifest) = super::tests::relocate(chain, &server);
    let effects = Arc::new(Effects {
        manifest: manifest.clone(),
        dns: AtomicUsize::new(0),
    });
    let transport = SearchTransport::new(
        manifest,
        effects.clone(),
        Arc::new(
            TransportBudget::new(Duration::from_secs(5), 1, 1_048_576).expect("budget"),
        ),
        effects.clone(),
    )
    .expect("transport");

    assert!(matches!(
        chain
            .execute_with_reachability(
                &transport,
                Reachability {
                    duckduckgo: false,
                    bing: false,
                },
            )
            .await,
        Err(CompatibilityChainError::Exhausted(_))
    ));
    assert_eq!(server.count(), 0);
    assert_eq!(effects.dns.load(Ordering::Relaxed), 0);
}
