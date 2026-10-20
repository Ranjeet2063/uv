use http::HeaderValue;
use uv_distribution_types::{IndexLocations, IndexUrl, RegistryBuiltWheel};
use uv_redacted::SafeUrl;

use crate::{Error, ErrorKind};

/// A registry wheel request with proxy mapping and metadata cache settings applied.
///
/// Construct this through [`Self::for_wheel`] before fetching registry wheel metadata.
#[derive(Debug, Clone)]
pub(super) struct RegistryWheelRequest<'a> {
    url: SafeUrl,
    effective_index: &'a IndexUrl,
    cache_control: Option<HeaderValue>,
}

impl<'a> RegistryWheelRequest<'a> {
    /// Resolve the wheel's artifact URL, effective index, and metadata cache override together.
    pub(super) fn for_wheel(
        wheel: &'a RegistryBuiltWheel,
        indexes: &'a IndexLocations,
    ) -> Result<Self, Error> {
        let (url, effective_index) = if let Some(route) = indexes.proxy_route_for(&wheel.index) {
            let url = route
                .artifact_url_for_request(&wheel.file.url)
                .map_err(ErrorKind::ProxyIndex)?;
            (url, route.effective_url())
        } else {
            let url = wheel.file.url.to_url().map_err(ErrorKind::InvalidUrl)?;
            (url, &wheel.index)
        };
        // PEP 658 metadata uses the original index's policy. Reading metadata from a wheel uses
        // the effective endpoint's policy, where shared URLs follow index declaration order.
        let policy_index = if wheel.file.dist_info_metadata.is_some() {
            &wheel.index
        } else {
            effective_index
        };
        let cache_control = indexes
            .request_policy(policy_index)
            .artifact_cache_control();
        Ok(Self {
            url,
            effective_index,
            cache_control,
        })
    }

    /// Borrow the URL used for requests.
    pub(super) fn as_url(&self) -> &SafeUrl {
        &self.url
    }

    /// Consume the prepared URL when making a request.
    pub(super) fn into_url(self) -> SafeUrl {
        self.url
    }

    /// Consume the request into its URL, cache and capability index, and cache override.
    pub(super) fn into_parts(self) -> (SafeUrl, &'a IndexUrl, Option<HeaderValue>) {
        (self.url, self.effective_index, self.cache_control)
    }
}
