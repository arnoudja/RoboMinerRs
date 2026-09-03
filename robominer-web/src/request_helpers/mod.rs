//! Form/query parsing and login redirect helpers.

mod form;
mod redirect;

pub(crate) use form::{is_post, mutation_form_has, mutation_i64, query_i64, query_signed_i64};
pub(crate) use redirect::{
    auth_page_href, encode_query_component, login_redirect, request_user_id, session_username,
    valid_login_return_to,
};

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::http::split_target;
    use crate::session::format_authenticated_cookie;

    use super::form::form_i64;
    use super::redirect::login_return_to_from_request;
    use super::{
        auth_page_href, encode_query_component, is_post, login_redirect, mutation_form_has,
        mutation_i64, query_i64, request_user_id, valid_login_return_to,
    };
    use crate::Request;

    fn request(path: &str) -> Request {
        let (path, query) = split_target(path);
        Request {
            method: "GET".to_string(),
            path,
            query,
            form: HashMap::new(),
            form_values: HashMap::new(),
            headers: HashMap::new(),
        }
    }

    fn post_form(path: &str, form: HashMap<String, String>) -> Request {
        let mut request = request(path);
        request.method = "POST".to_string();
        request.form_values = form
            .iter()
            .map(|(name, value)| (name.clone(), vec![value.clone()]))
            .collect();
        request.form = form;
        request
    }

    fn request_with_cookie(path: &str, cookie: &str) -> Request {
        let mut request = request(path);
        request
            .headers
            .insert("cookie".to_string(), cookie.to_string());
        request
    }

    #[test]
    fn query_parsing_decodes_parameters() {
        let request = request("/activity?rallyResultId=12&name=Robo+Miner%21");

        assert_eq!(request.path, "/activity");
        assert_eq!(query_i64(&request, "rallyResultId"), Some(12));
        assert_eq!(request.query.get("name"), Some(&"Robo Miner!".to_string()));
    }

    #[test]
    fn login_redirect_preserves_return_to_for_protected_routes() {
        let response = login_redirect(&request("/shop?selectedRobotPartTypeId=3"));
        assert_eq!(response.status, 302);
        assert!(response.headers.iter().any(|(name, value)| {
            *name == "Location" && value == "login?returnTo=shop%3FselectedRobotPartTypeId%3D3"
        }));
    }

    #[test]
    fn login_redirect_omits_return_to_for_root_and_auth_routes() {
        let root = login_redirect(&request("/"));
        assert!(
            root.headers
                .iter()
                .any(|(name, value)| *name == "Location" && value == "login")
        );

        let login = login_redirect(&request("/login"));
        assert!(
            login
                .headers
                .iter()
                .any(|(name, value)| *name == "Location" && value == "login")
        );
    }

    #[test]
    fn valid_login_return_to_rejects_protocol_relative_and_backslash_paths() {
        assert_eq!(valid_login_return_to("//evil.com"), None);
        assert_eq!(valid_login_return_to("/shop"), None);
        assert_eq!(valid_login_return_to(r"shop\admin"), None);
    }

    #[test]
    fn valid_login_return_to_rejects_external_and_auth_paths() {
        assert_eq!(
            valid_login_return_to("miningResults?rallyResultId=12"),
            Some("miningResults?rallyResultId=12")
        );
        assert_eq!(valid_login_return_to("shop"), Some("shop"));
        assert_eq!(valid_login_return_to("https://evil.test"), None);
        assert_eq!(valid_login_return_to("https:evil.com"), None);
        assert_eq!(valid_login_return_to("/shop"), None);
        assert_eq!(valid_login_return_to("login"), None);
        assert_eq!(valid_login_return_to("login?returnTo=shop"), None);
        assert_eq!(valid_login_return_to("logoff"), None);
        assert_eq!(valid_login_return_to("notARealPage"), None);
        assert_eq!(valid_login_return_to("notARealPage?x=1"), None);
    }

    #[test]
    fn login_return_to_from_request_builds_stable_query_strings() {
        assert_eq!(
            login_return_to_from_request(&request("/robot?robotId=2&tab=program")),
            Some("robot?robotId=2&tab=program".to_string())
        );
    }

    #[test]
    fn auth_page_href_preserves_signup_and_return_to() {
        assert_eq!(auth_page_href(false, None), "login");
        assert_eq!(
            auth_page_href(true, Some("shop?selectedRobotPartTypeId=3")),
            "login?signup=1&returnTo=shop%3FselectedRobotPartTypeId%3D3"
        );
    }

    #[test]
    fn encode_query_component_percent_encodes_spaces() {
        assert_eq!(encode_query_component("a b"), "a%20b");
    }

    #[test]
    fn user_id_is_read_from_signed_session_cookie_only() {
        assert_eq!(request_user_id(&request("/miningResults?userId=42")), None);
        assert_eq!(
            request_user_id(&request_with_cookie(
                "/miningResults",
                &format_authenticated_cookie(77, "Player")
            )),
            Some(77)
        );
    }

    #[test]
    fn mutation_helpers_require_post_form_not_query() {
        let get_query = request("/shop?buyRobotPartId=9");
        assert!(!is_post(&get_query));
        assert_eq!(query_i64(&get_query, "buyRobotPartId"), Some(9));
        assert_eq!(mutation_i64(&get_query, "buyRobotPartId"), None);
        assert!(!mutation_form_has(&get_query, "sellAllUnassigned"));

        let get_with_form = {
            let mut request = request("/shop");
            request
                .form
                .insert("buyRobotPartId".to_string(), "9".to_string());
            request
                .form
                .insert("sellAllUnassigned".to_string(), "1".to_string());
            request
        };
        assert_eq!(form_i64(&get_with_form, "buyRobotPartId"), Some(9));
        assert_eq!(mutation_i64(&get_with_form, "buyRobotPartId"), None);
        assert!(!mutation_form_has(&get_with_form, "sellAllUnassigned"));

        let mut form = HashMap::new();
        form.insert("buyRobotPartId".to_string(), "9".to_string());
        form.insert("sellAllUnassigned".to_string(), "1".to_string());
        let post = post_form("/shop", form);
        assert!(is_post(&post));
        assert_eq!(mutation_i64(&post, "buyRobotPartId"), Some(9));
        assert!(mutation_form_has(&post, "sellAllUnassigned"));
    }
}
