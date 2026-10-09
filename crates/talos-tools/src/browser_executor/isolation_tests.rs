//! Deterministic document ownership fixture; not evidence for a native browser driver.

use super::*;
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct Document {
    frame: BrowserFrameRef,
    text: &'static str,
    sensitive: &'static str,
    children: Vec<Document>,
}

impl Document {
    fn find(&self, frame: &BrowserFrameRef) -> Option<&Self> {
        if &self.frame == frame {
            Some(self)
        } else {
            self.children.iter().find_map(|child| child.find(frame))
        }
    }

    fn paint(&self, pixels: &mut RgbaImage, offset: u32) {
        pixels.put_pixel(offset, 0, Rgba([255, 0, 0, 255]));
        for (index, child) in self.children.iter().enumerate() {
            child.paint(pixels, offset + index as u32 + 1);
        }
    }
}

struct DocumentExecutor {
    root: Document,
    containment_known: bool,
    executions: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl BrowserExecutor for DocumentExecutor {
    fn supports(&self, request: &BrowserRequest, _: &BrowserPermissionTarget) -> bool {
        self.containment_known
            && matches!(
                request.operation(),
                BrowserOperation::Read | BrowserOperation::Snapshot | BrowserOperation::Screenshot
            )
    }

    async fn execute(
        &mut self,
        request: &BrowserRequest,
        target: &BrowserPermissionTarget,
        context: &mut BrowserContext,
    ) -> BrowserOutput {
        self.executions.fetch_add(1, Ordering::SeqCst);
        let BrowserPermissionTarget::Observe { scope, .. } = target else {
            panic!("fixture expects an observation");
        };
        let document = self.root.find(&scope.frame).expect("fixture document");
        assert!(!document.sensitive.is_empty());
        match request.operation() {
            BrowserOperation::Read => BrowserOutput::Read {
                text: document.text.into(),
                truncated: false,
            },
            BrowserOperation::Snapshot => {
                let identity = context
                    .replace_snapshot(&scope.tab, &scope.frame, 1)
                    .expect("snapshot");
                BrowserOutput::Snapshot {
                    snapshot_ref: identity.snapshot,
                    nodes: vec![BrowserSnapshotNode {
                        element_ref: identity.elements[0].clone(),
                        parent_element_ref: None,
                        role: BrowserRole::Statictext,
                        name: document.text.into(),
                        states: Vec::new(),
                    }],
                    truncated: false,
                }
            }
            BrowserOperation::Screenshot => {
                let mut pixels = RgbaImage::from_pixel(8, 1, Rgba([0, 0, 0, 255]));
                document.paint(&mut pixels, 0);
                // All descendant pixels are removed before publishing the capability.
                for x in 1..8 {
                    pixels.put_pixel(x, 0, Rgba([0, 0, 0, 255]));
                }
                let mut png = Cursor::new(Vec::new());
                DynamicImage::ImageRgba8(pixels)
                    .write_to(&mut png, ImageFormat::Png)
                    .expect("PNG");
                let bytes = png.into_inner();
                let byte_length = bytes.len() as u64;
                let artifact_ref = context.publish_screenshot(bytes, 8, 1).expect("artifact");
                BrowserOutput::Screenshot {
                    artifact_ref,
                    mime: BrowserScreenshotMime::Png,
                    width: 8,
                    height: 1,
                    byte_length,
                }
            }
            _ => unreachable!("admission restricts fixture operations"),
        }
    }
}

#[tokio::test]
async fn selected_document_excludes_same_cross_and_nested_child_content_and_pixels() {
    let mut context = BrowserContext::new();
    let tab = context.insert_tab().expect("tab");
    let origin =
        || Some(BrowserOrigin::from_effective_url("https://parent.example").expect("origin"));
    let root = context
        .insert_frame(&tab, None, origin(), true)
        .expect("root");
    let same = context
        .insert_frame(&tab, Some(&root), origin(), true)
        .expect("same");
    let cross = context
        .insert_frame(
            &tab,
            Some(&root),
            Some(BrowserOrigin::from_effective_url("https://child.example").expect("origin")),
            true,
        )
        .expect("cross");
    let nested = context
        .insert_frame(&tab, Some(&cross), origin(), true)
        .expect("nested");
    let document = Document {
        frame: root.clone(),
        text: "parent public",
        sensitive: "parent password",
        children: vec![
            Document {
                frame: same.clone(),
                text: "same secret",
                sensitive: "same OTP",
                children: vec![],
            },
            Document {
                frame: cross.clone(),
                text: "cross secret",
                sensitive: "cross password",
                children: vec![Document {
                    frame: nested,
                    text: "nested secret",
                    sensitive: "nested OTP",
                    children: vec![],
                }],
            },
        ],
    };
    let mut unmasked = RgbaImage::from_pixel(8, 1, Rgba([0, 0, 0, 255]));
    document.paint(&mut unmasked, 0);
    assert!(unmasked.pixels().skip(1).any(|pixel| pixel[0] == 255));
    let mut host = BrowserHost::new(DocumentExecutor {
        root: document,
        containment_known: true,
        executions: Arc::new(AtomicUsize::new(0)),
    });
    *host.context_mut() = context;
    for frame in [&root, &same, &cross] {
        for operation in ["read", "snapshot", "screenshot"] {
            let raw = serde_json::json!({"protocolVersion":2,"operation":operation,"tabRef":tab,"frameRef":frame}).to_string();
            let invocation = host.prepare(&raw).expect("admission");
            let resource = host.permission_resource(&invocation).expect("resource");
            let authorization = ExactBrowserPermissionEvaluator::for_resource(resource.clone())
                .evaluate(&resource)
                .expect("approval");
            let output = host
                .execute_prepared(invocation, authorization, std::future::pending())
                .await
                .expect("output");
            assert!(!output.model_json().contains("password"));
            assert!(!output.model_json().contains("OTP"));
            assert!(!output.model_json().contains("nested secret"));
            if frame == &root {
                assert!(!output.model_json().contains("same secret"));
                assert!(!output.model_json().contains("cross secret"));
            }
            if let BrowserOutput::Screenshot { artifact_ref, .. } =
                serde_json::from_str(output.model_json()).expect("typed output")
            {
                let (bytes, _) = host
                    .consume_screenshot(&artifact_ref, &resource)
                    .expect("consume");
                let pixels = image::load_from_memory(&bytes)
                    .expect("decoded PNG")
                    .to_rgba8();
                assert_eq!(pixels.get_pixel(0, 0)[0], 255);
                assert!(pixels.pixels().skip(1).all(|pixel| pixel[0] == 0));
            }
        }
    }
}

#[tokio::test]
async fn unknown_containment_refuses_before_approval_and_executor_dispatch() {
    let mut context = BrowserContext::new();
    let tab = context.insert_tab().expect("tab");
    let frame = context
        .insert_frame(
            &tab,
            None,
            Some(BrowserOrigin::from_effective_url("https://parent.example").expect("origin")),
            true,
        )
        .expect("frame");
    let executions = Arc::new(AtomicUsize::new(0));
    let mut host = BrowserHost::new(DocumentExecutor {
        root: Document {
            frame: frame.clone(),
            text: "private page text",
            sensitive: "private password",
            children: Vec::new(),
        },
        containment_known: false,
        executions: executions.clone(),
    });
    *host.context_mut() = context;
    let mut approvals = 0;
    for operation in ["read", "snapshot", "screenshot"] {
        let raw = serde_json::json!({"protocolVersion":2,"operation":operation,"tabRef":tab,"frameRef":frame}).to_string();
        let error = match host.prepare(&raw) {
            Ok(invocation) => {
                approvals += 1;
                let resource = host.permission_resource(&invocation).expect("resource");
                let authorization = ExactBrowserPermissionEvaluator::for_resource(resource.clone())
                    .evaluate(&resource)
                    .expect("approval");
                host.execute_prepared(invocation, authorization, std::future::pending())
                    .await
                    .err()
                    .expect("must refuse")
            }
            Err(error) => error,
        };
        assert!(matches!(error, BrowserHostError::Rejected { code: BrowserFailureCode::UnsupportedOperation, .. }));
        let output = error
            .to_prepared(Some(match operation {
                "read" => BrowserOperation::Read,
                "snapshot" => BrowserOperation::Snapshot,
                _ => BrowserOperation::Screenshot,
            }))
            .into_output();
        let value: serde_json::Value =
            serde_json::from_str(&output.result.content).expect("failure");
        assert_eq!(value["code"], "UnsupportedOperation");
        assert_eq!(value["outcome"], "notExecuted");
        assert!(!output.result.content.contains("private"));
    }
    assert_eq!(approvals, 0);
    assert_eq!(executions.load(Ordering::SeqCst), 0);
}
