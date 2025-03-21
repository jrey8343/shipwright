use shipwright_db::entities::appointment::Appointment;

pub struct RefreshDataJob {
    pub client: reqwest::Client,
}
pub async fn refresh_nookal_data(app_state: &AppState) -> Result<(), Error> {
    // Check what page the database is synced to
    let synced_count = Appointment::get_current_count(&app_state.db_pool).await?;
    let synced_page = Appointment::get_current_page(synced_count, 100)?; // Last synced page

    // Fetch the latest total count from the external system
    let AppointmentsResponse { details, .. } =
        AppointmentsResponse::fetch(&app_state.reqwest_client, synced_page).await?;

    let current_count = details.total_items;
    let current_page = current_count / 100; // Round up to get total pages

    // Ensure we only fetch new records
    if current_count > synced_count {
        for page in synced_page..=current_page {
            let AppointmentsResponse { data, .. } =
                AppointmentsResponse::fetch(&app_state.reqwest_client, page).await?;

            let appointments_to_insert = if page == synced_page {
                let start_index = synced_count % 100;
                &data.results.appointments[start_index as usize..].to_vec()
            } else {
                &data.results.appointments
            };

            Appointment::create_batch(appointments_to_insert, &app_state.db_pool).await?;

            info!(
                "✅ Synced {} appointments from page {}",
                data.results.appointments.len(),
                page
            );
        }
    } else {
        info!("✅ No new appointments to sync. Database is up-to-date.");
    }

    Ok(())
}
