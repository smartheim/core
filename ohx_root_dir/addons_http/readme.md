# Addons static file serving

An Addon may have arbitrary files at a /web directory within its software container.
That directory is mounted into this directory under `/addons_http/:addon_id/`
and served under `/addons/:addon_id/`.
