#[cfg(test)]
mod user_preferences_tests {
    use super::*;
    #[test]
    #[ignore = "Requires an installed printer named by ARTCRAFT_TEST_PRINTER; never submits"]
    fn user_preferences_merge_without_spooling() {
        let name = std::env::var("ARTCRAFT_TEST_PRINTER").unwrap();
        let device = PrinterDevice::all().unwrap().into_iter().find(|p| p.name() == name).unwrap();
        let ticket = user_driver_ticket(&device).unwrap();
        let mut builder = PrintTicketBuilder::new(&device).unwrap();
        builder.merge(ticket).unwrap();
        let validated = builder.build().unwrap();
        let xml = std::str::from_utf8(validated.get_xml()).unwrap();
        assert!(xml.contains("PrintTicket"));
        assert!(xml.contains("PageMedia"));
    }
}
