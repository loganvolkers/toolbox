//! `webhook`: fire and forget (docs/design/action-runs.md §5). The protocol `run` message is
//! POSTed to `url`, and a 2xx means only that it was delivered: the run is then checked by its
//! postcondition, or ends `unverified`, never `succeeded` on the webhook's word.

use super::{Context, Executor, Outcome, Preview};
use seikika_model::ci::Hosts;
use serde_json::{Value, json};
use std::sync::Arc;

/// A `webhook` executor.
#[derive(Debug, Clone)]
pub struct Webhook {
    pub url: String,
    /// The environment variable holding a bearer token, if one is sent.
    pub token_env: Option<String>,
}

pub(crate) fn parse(v: &Value, _: &Hosts) -> Result<Arc<dyn Executor>, String> {
    let s = |k: &str| v.get(k).and_then(Value::as_str).map(str::to_owned);
    Ok(Arc::new(Webhook {
        url: s("url").ok_or("executor.url is required")?,
        token_env: s("token"),
    }))
}

impl Executor for Webhook {
    fn kind(&self) -> &'static str {
        "webhook"
    }

    fn preview(&self, _: &Context) -> Result<Preview, String> {
        Ok(Preview {
            effects: vec![
                json!({"resource": format!("webhook:{}", self.url), "operation": "post"}),
            ],
            ..Preview::default()
        })
    }

    fn run(&self, ctx: &Context, confirmed: &[Value]) -> Outcome {
        let mut req = crate::http::client()
            .post(&self.url)
            .json(&super::protocol::run_message(ctx, confirmed));
        if let Some(var) = &self.token_env {
            match std::env::var(var) {
                Ok(t) => req = req.bearer_auth(t),
                Err(_) => return Outcome::failed(format!("set {var} to reach {}", self.url)),
            }
        }
        match crate::http::send(req) {
            Ok((st, _)) if st.is_success() => Outcome {
                status: "succeeded".into(),
                outputs: json!({"delivered": st.as_u16()}),
                error: None,
                progress: vec![],
                child: None,
            },
            Ok((st, text)) => {
                Outcome::failed(format!("{} answered {st}: {}", self.url, text.trim()))
            }
            Err(e) => Outcome::failed(format!("{}: {e}", self.url)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::fake;
    use serde_json::Map;

    fn ctx() -> Context {
        Context {
            run_id: Some("run_3".into()),
            portal: "org".into(),
            action: "notify".into(),
            action_version: "v1".into(),
            entity: json!({"id": "service:pay"}),
            inputs: Map::new(),
            actor: "user:ann".into(),
            approved_by: vec![],
            related: Map::new(),
            grant: None,
            cwd: std::env::temp_dir(),
        }
    }

    #[test]
    fn delivers_the_run_message_and_fails_on_a_refusal() {
        let (url, seen) = fake::serve("", vec![("POST /hook", 202, json!({}))]);
        let w = parse(&json!({"url": format!("{url}/hook")}), &Hosts::default()).unwrap();
        let out = w.run(&ctx(), &[]);
        assert_eq!(out.status, "succeeded", "{:?}", out.error);
        assert_eq!(out.outputs, json!({"delivered": 202}));
        let body: Value = serde_json::from_str(&seen.lock().unwrap()[0].2).unwrap();
        assert!(
            seikika_model::schema::executor_messages().is_valid(&body),
            "{body}"
        );
        let w = parse(
            &json!({"url": format!("{url}/elsewhere")}),
            &Hosts::default(),
        )
        .unwrap();
        assert_eq!(w.run(&ctx(), &[]).status, "failed");
        let w = parse(
            &json!({"url": url, "token": "SEIKIKA_TEST_NO_SUCH_VAR"}),
            &Hosts::default(),
        )
        .unwrap();
        assert!(
            w.run(&ctx(), &[])
                .error
                .unwrap()
                .contains("SEIKIKA_TEST_NO_SUCH_VAR")
        );
    }
}
