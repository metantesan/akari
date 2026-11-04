pub(crate) fn system_locales() -> Vec<String> {
    sys_locale::get_locales().collect()
}
