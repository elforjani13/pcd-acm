use chrono::Utc;
use hl7::messages::ORU_R01;
use hl7::segments::{NTE, OBX, ORC, PID, PV1};

#[derive(Debug)]
#[allow(dead_code)]
pub struct PCD04Message {
    heartbeat_ar_type: &'static str,
    oru_r40: ORU_R01,
    msg_control_id_iter: usize,
    obx_count: usize,
    equip_ii: String,
}
#[allow(dead_code)]
impl PCD04Message {
    const ACTOR_EUI64: &'static str = "0000000000000001^EUI-64";
    const ACTOR_EUI64_SUB: &'static str = "0000000000000001&EUI-64";
    const ACCEPT_ACK_TYPE_ACM: &'static str = "AL";
    const APP_ACK_TYPE: &'static str = "NE";

    pub(crate) fn new() -> Self {
        PCD04Message {
            heartbeat_ar_type: "",
            oru_r40: ORU_R01::default(),
            msg_control_id_iter: 0,
            obx_count: 0,
            equip_ii: String::new(), // ntf
        }
    }

    pub fn get_message(&self) -> Option<ORU_R01> {
        Some(self.oru_r40.clone())
    }

    pub fn create_pcd04_message(
        &mut self,
        assigned_location: &str,
        equip_ii: &str,
        patient_id_list: &str,
        patient_name: &str,
        patient_dob: &str,
        patient_sex: &str,
        alert_type: &str,
        alert_text: &str,
        alert_phase: &str,
        alert_kind_prio_star: &str,
        src_containment_tree_id: &str,
        obs_type: &str,
        obs_value: &str,
        obs_value_type: &str,
        obs_unit: &str,
        unique_alert_uuid: &str,
        alert_kind: &str,
        obs_det_time: &str,
        alert_counter: i32,
        alert_state: &str,
        alert_inactivation_state: &str,
        sending_facility: &str,
        receiving_app: Option<&str>,
        processing_id: &str,
        mds_type: &str,
        vmd_type: &str,
    ) {
        let msg_time = Utc::now();
        let msg_time_str = msg_time.format("%Y%m%d%H%M%S%Z").to_string();

        self.create_msh_segment_acm(
            &msg_time_str,
            sending_facility,
            receiving_app,
            processing_id,
        );
        self.create_pid_segment_acm(patient_id_list, patient_name, patient_dob, patient_sex);
        self.create_pv1_segment_acm(assigned_location);
        self.equip_ii = equip_ii.to_string();
        self.create_obr_segment_acm(&msg_time_str, unique_alert_uuid, alert_counter);
        self.obx_count = 0;

        let mds_tree = format!("{}.0.0", src_containment_tree_id.split(".").next().unwrap());
        let vmd_tree = format!(
            "{}.{}.0",
            src_containment_tree_id.split(".").next().unwrap(),
            src_containment_tree_id.split(".").nth(1).unwrap()
        );

        self.create_obx_segment_acm(0, mds_type, "", "", "", "", "", &mds_tree);
        self.create_obx_segment_acm(0, vmd_type, "", "", "", "", "", &vmd_tree);
        self.create_obx_segment_acm(
            1,
            alert_type,
            alert_text,
            "",
            "",
            "",
            "",
            src_containment_tree_id,
        );
        self.create_obx_segment_acm(
            2,
            obs_type,
            obs_value,
            obs_unit,
            obs_det_time,
            "",
            obs_value_type,
            src_containment_tree_id,
        );
        self.create_obx_segment_acm(
            3,
            "68481^MDC_ATTR_EVENT_PHASE^MDC",
            alert_phase,
            "",
            "",
            "",
            "",
            src_containment_tree_id,
        );
        self.create_obx_segment_acm(
            4,
            "68482^MDC_ATTR_ALARM_STATE^MDC",
            alert_state,
            "",
            "",
            "",
            "",
            src_containment_tree_id,
        );
        self.create_obx_segment_acm(
            5,
            "68483^MDC_ATTR_ALARM_INACTIVATION_STATE^MDC",
            alert_inactivation_state,
            "",
            "",
            "",
            "",
            src_containment_tree_id,
        );
        self.create_obx_segment_acm(
            6,
            "68484^MDC_ATTR_ALARM_PRIORITY^MDC",
            alert_kind_prio_star,
            "",
            "",
            "",
            "",
            src_containment_tree_id,
        );
        self.create_obx_segment_acm(
            7,
            "68485^MDC_ATTR_ALERT_TYPE^MDC",
            alert_kind,
            "",
            "",
            "",
            "",
            src_containment_tree_id,
        );
    }

    pub fn append_watchdog_obx_segment(
        &mut self,
        timeout_period: &str,
        timeout_unit: &str,
        mds_tree: &str,
    ) {
        // let actual_timeout_unit = Some(timeout_unit)
        //     .filter(|unit| !unit.is_empty())
        //     .unwrap_or("264320^MDC_DIM_SEC^MDC");

        self.create_obx_segment_acm(
            8,
            "67860^MDC_ATTR_CONFIRM_TIMEOUT^MDC",
            timeout_period,
            timeout_unit,
            "",
            "",
            "NM",
            mds_tree,
        );
    }

    fn create_msh_segment_acm(
        &mut self,
        msg_time_str: &str,
        sending_facility: &str,
        receiving_app: Option<&str>,
        processing_id: &str,
    ) {
        let msg_control_id_val = self.msg_control_id_iter.to_string();
        self.msg_control_id_iter += 1;

        let msh = &mut self.oru_r40.msh;
        msh.msh_3_sending_application = Some(Self::ACTOR_EUI64.to_string());
        msh.msh_4_sending_facility = Some(sending_facility.to_string());

        if let Some(receiving_app) = receiving_app {
            msh.msh_5_receiving_application = Some(receiving_app.to_string());
        }
        msh.msh_7_date_time_of_message = msg_time_str.to_string();
        msh.msh_9_message_type = "ORU^R40^ORU_R40".to_string();
        msh.msh_10_message_control_id = msg_control_id_val.to_string();
        msh.msh_11_processing_id = processing_id.to_string();
        msh.msh_15_accept_acknowledgment_type = Some(Self::ACCEPT_ACK_TYPE_ACM.to_string());
        msh.msh_16_application_acknowledgment_type = Some(Self::APP_ACK_TYPE.to_string());
        msh.msh_21_message_profile_identifier = Some(vec![
            "IHE_PCD_ACM_001^IHE PCD^1.3.6.1.4.1.19376.1.6.1.4.1^ISO".to_string(),
        ]);
    }

    fn create_pid_segment_acm(
        &mut self,
        patient_id_list: &str,
        patient_name: &str,
        patient_dob: &str,
        patient_sex: &str,
    ) {
        let mut pid = PID::default();

        pid.pid3_patient_identifier_list = vec![patient_id_list.to_string()];
        pid.pid5_patient_name = vec![patient_name.to_string()];
        pid.pid7_date_time_of_birth = Some(patient_dob.to_string());
        pid.pid8_administrative_sex = Some(patient_sex.to_string());

        self.oru_r40
            .oru_r01_patient_result
            .iter_mut()
            .flat_map(|patient_result| &mut patient_result.oru_r01_patient)
            .for_each(|patient| patient.pid = pid.clone());
    }

    fn create_pv1_segment_acm(&mut self, location: &str) {
        let mut pv1 = PV1::default();
        pv1.pv1_2_patient_class = "I".to_string();
        pv1.pv1_3_assigned_patient_location = Some(location.to_string());

        for patient_result in &mut self.oru_r40.oru_r01_patient_result {
            if let Some(patient) = &mut patient_result.oru_r01_patient {
                if let Some(visit) = &mut patient.oru_r01_visit {
                    visit.pv1 = pv1.clone();
                }
            }
        }
    }

    fn inc_alert_counter(&mut self) {
        let mut orc = ORC::default();
        if let Some(old_count_str) = orc
            .orc_3_filler_order_number
            .as_deref()
            .map(|s| s.split('^').collect::<Vec<_>>())
        {
            if let Some(old_count) = old_count_str[0].parse::<i32>().ok() {
                let filler_order_number = format!(
                    "{}^{}^{}",
                    old_count + 1,
                    old_count_str[1],
                    old_count_str[2]
                );

                orc.orc_3_filler_order_number = Some(filler_order_number);

                if let Some(obx_segment) = self.oru_r40.oru_r01_patient_result[0]
                    .oru_r01_patient_observation
                    .get_mut(2)
                {
                    if let Some(obx_5_observation_value) =
                        &mut obx_segment.obx.obx_5_observation_value
                    {
                        obx_5_observation_value[0] = "continue".to_string();
                    }
                }
            }
        }
    }

    pub fn set_control_id(&mut self, id: &str) {
        let msg_ctrl_id = &mut self.oru_r40.msh;
        msg_ctrl_id.msh_10_message_control_id = id.to_string();
    }

    fn set_observation_value_by_index(&mut self, index: usize, observation_value: &str) {
        self.oru_r40
            .oru_r01_patient_result
            .iter_mut()
            .flat_map(|res| &mut res.oru_r01_patient_observation)
            .enumerate()
            .for_each(|(obs_index, result)| {
                if obs_index == index {
                    let mut obx_seg = OBX::default();
                    obx_seg.obx_5_observation_value = Some(vec![observation_value.to_string()]);
                    result.obx = obx_seg;
                }
            });
    }

    fn set_alarm_type_and_text(&mut self, alarm_type: &str, alarm_text: &str) {
        self.oru_r40
            .oru_r01_patient_result
            .iter_mut()
            .flat_map(|result| &mut result.oru_r01_patient_observation)
            .for_each(|patient| {
                patient.obx = {
                    let mut obx_seg1 = OBX::default();
                    obx_seg1.obx_3_observation_identifier = alarm_type.to_string();
                    obx_seg1.obx_5_observation_value = Some(vec![alarm_text.to_string()]);
                    obx_seg1
                }
            })
    }

    fn set_alarm_value(&mut self, value: f64, obs_type: &str, unit: &str, time: &str) {
        self.oru_r40
            .oru_r01_patient_result
            .iter_mut()
            .flat_map(|res| &mut res.oru_r01_patient_observation)
            .for_each(|patient| {
                let mut obx_seg2 = OBX::default();
                obx_seg2.obx_2_value_type = value.to_string();
                obx_seg2.obx_5_observation_value = Some(vec![obs_type.to_string()]);
                obx_seg2.obx_6_units = Some(unit.to_string());
                obx_seg2.obx_14_date_time_of_the_observation = Some(time.to_string());

                patient.obx = obx_seg2
            })
    }

    fn set_alarm_ctp(&mut self, nte: NTE) {
        self.oru_r40
            .oru_r01_patient_result
            .iter_mut()
            .for_each(|result| {
                result
                    .oru_r01_patient
                    .iter_mut()
                    .for_each(|p| p.nte = Some(vec![nte.clone()]));
            })
    }

    #[allow(dead_code)]
    fn set_alarm_id(&mut self, _alarm_id: &str) {}

    fn set_alarm_phase(&mut self, alert_phase: &str) {
        self.set_observation_value_by_index(3, alert_phase)
    }

    fn set_alarm_state(&mut self, alert_state: &str) {
        self.set_observation_value_by_index(4, alert_state)
    }

    fn set_alarm_inactivation_state(&mut self, alert_state: &str) {
        self.set_observation_value_by_index(5, alert_state);
    }

    fn set_alarm_prio(&mut self, alert_prio: &str) {
        self.set_observation_value_by_index(6, alert_prio);
    }

    fn set_alarm_kind(&mut self, alert_kind: &str) {
        self.set_observation_value_by_index(7, alert_kind);
    }

    fn get_device_id(&self) -> Option<&str> {
        self.oru_r40
            .oru_r01_patient_result
            .iter()
            .flat_map(|result| &result.oru_r01_patient_observation)
            .flat_map(|r| r.obx.obx_18_equipment_instance_identifier.as_deref())
            .flat_map(|s| s.get(0).map(|s| s.as_str()))
            .next()
    }

    fn get_location(&self) -> Option<&str> {
        for location in &self.oru_r40.oru_r01_patient_result {
            if let Some(p) = &location.oru_r01_patient {
                if let Some(visit) = &p.oru_r01_visit {
                    return visit.pv1.pv1_3_assigned_patient_location.as_deref();
                }
            }
        }
        None
    }
    fn get_equip(&self) -> Option<&str> {
        for result in &self.oru_r40.oru_r01_patient_result {
            for obs in &result.oru_r01_patient_observation {
                if let Some(obx) = Option::from(&obs.obx) {
                    return obx
                        .obx_18_equipment_instance_identifier
                        .as_deref()
                        .and_then(|v| v.first().map(|s| s.as_str()));
                }
            }
        }
        None
    }

    fn get_patient_id(&self) -> Option<&str> {
        for patient_result in &self.oru_r40.oru_r01_patient_result {
            for patient in &patient_result.oru_r01_patient {
                return patient
                    .pid
                    .pid3_patient_identifier_list
                    .get(0)
                    .map(|s| s.as_str());
            }
        }
        None
    }

    fn get_patient_name(&self) -> Option<&str> {
        for patient_result in &self.oru_r40.oru_r01_patient_result {
            for patient in &patient_result.oru_r01_patient {
                return patient.pid.pid5_patient_name.first().map(|s| s.as_str());
            }
        }
        None
    }

    fn get_patient_dob(&self) -> Option<&str> {
        for patient_result in &self.oru_r40.oru_r01_patient_result {
            for patient in &patient_result.oru_r01_patient {
                return patient.pid.pid7_date_time_of_birth.as_deref();
            }
        }
        None
    }

    fn get_patient_sex(&self) -> Option<&str> {
        for patient_result in &self.oru_r40.oru_r01_patient_result {
            for patient in &patient_result.oru_r01_patient {
                return patient.pid.pid8_administrative_sex.as_deref();
            }
        }
        None
    }

    fn get_obx_segment(&self, nr: &str) -> Option<&str> {
        for patient_result in &self.oru_r40.oru_r01_patient_result {
            for patient_obs in &patient_result.oru_r01_patient_observation {
                if patient_obs.obx.obx_1_set_id == Some(nr.to_string()) {
                    return patient_obs.obx.obx_1_set_id.as_deref();
                }
            }
        }
        None
    }

    fn create_obr_segment_acm(
        &mut self,
        message_time_str: &str,
        unique_alert_uuid: &str,
        alert_update: i32,
    ) {
        let mut obr = ORC::default();
        let filler_order_number = format!(
            "{}^{}^{}",
            alert_update,
            unique_alert_uuid,
            Self::ACTOR_EUI64
        );

        obr.orc_1_order_control = "1".to_string();
        obr.orc_3_filler_order_number = Some(filler_order_number);
        obr.orc_4_placer_group_number = Some("196616^MDC_EVT_ALARM^MDC".to_string());
        obr.orc_7_quantity_timing = Some(vec![message_time_str.to_string()]);

        if alert_update > 0 {
            let parent_alert = format!("^0&{}&{}", unique_alert_uuid, Self::ACTOR_EUI64_SUB);
            obr.orc_29_order_type = Some(parent_alert.to_string())
        }
    }

    fn create_obx_segment_acm(
        &mut self,
        set_id: usize,
        obs_id: &str,
        obs_value: &str,
        obs_unit: &str,
        obs_time_str: &str,
        obs_site: &str,
        obs_value_type: &str,
        ctp: &str,
    ) {
        let mut obx = OBX::default();

        self.obx_count += 1;

        obx.obx_1_set_id = Some(self.obx_count.to_string());

        if !obs_value_type.is_empty() {
            obx.obx_2_value_type = obs_value_type.to_string();
            obx.obx_11_observation_result_status = "F".to_string();
        } else {
            obx.obx_11_observation_result_status = "X".to_string();
        }

        obx.obx_3_observation_identifier = obs_id.to_string();
        obx.obx_4_observation_sub_id = format!("{}.{}", ctp, set_id);
        obx.obx_5_observation_value = Some(vec![obs_value.to_string()]);
        obx.obx_6_units = Some(obs_unit.to_string());
        obx.obx_14_date_time_of_the_observation = Some(obs_time_str.to_string());
        obx.obx_18_equipment_instance_identifier = Some(vec![self.equip_ii.to_string()]);
        obx.obx_20_observation_site = Some(vec![obs_site.to_string()]);
    }
}
