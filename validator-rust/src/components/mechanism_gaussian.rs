use crate::errors::*;


use std::collections::HashMap;


use crate::components::Aggregator;
use crate::{proto, base};

use crate::components::{Component, Expandable};
use crate::base::{Value, NodeProperties, ArrayND, get_constant, Sensitivity, prepend, ValueProperties};
use ndarray::Array;

impl Component for proto::GaussianMechanism {
    // modify min, max, n, categories, is_public, non-null, etc. based on the arguments and component
    fn propagate_property(
        &self,
        privacy_definition: &proto::PrivacyDefinition,
        _public_arguments: &HashMap<String, Value>,
        properties: &base::NodeProperties,
    ) -> Result<ValueProperties> {
        let mut data_property = properties.get("data")
            .ok_or("data: missing")?.get_arraynd()
            .map_err(prepend("data:"))?.clone();

        let aggregator = data_property.aggregator.clone()
            .ok_or::<Error>("aggregator: missing".into())?;

        // sensitivity must be computable
        aggregator.component.compute_sensitivity(
            &privacy_definition,
            &aggregator.properties,
            &Sensitivity::KNorm(2))?;

        data_property.releasable = true;
        Ok(data_property.into())
    }

    fn get_names(
        &self,
        _properties: &NodeProperties,
    ) -> Result<Vec<String>> {
        Err("get_names not implemented".into())
    }
}


impl Expandable for proto::GaussianMechanism {
    fn expand_component(
        &self,
        privacy_definition: &proto::PrivacyDefinition,
        component: &proto::Component,
        properties: &base::NodeProperties,
        component_id: u32,
        maximum_id: u32,
    ) -> Result<(u32, HashMap<u32, proto::Component>)> {
        let mut current_id = maximum_id.clone();
        let mut graph_expansion: HashMap<u32, proto::Component> = HashMap::new();

        // TODO: SECURITY: a user must not be able to define this directly
        if !properties.contains_key("sensitivity") {
            // sensitivity literal
            let mut data_property = properties.get("data")
                .ok_or("data: missing")?.get_arraynd()
                .map_err(prepend("data:"))?.clone();

            let aggregator = data_property.aggregator.clone()
                .ok_or::<Error>("aggregator: missing".into())?;

            let sensitivity: Value = Array::from(aggregator.component
                .compute_sensitivity(privacy_definition, &aggregator.properties, &Sensitivity::KNorm(2))?).into_dyn().into();
            current_id += 1;
            let id_sensitivity = current_id.clone();
            graph_expansion.insert(id_sensitivity, get_constant(&sensitivity, &component.batch));

            // noising
            let mut noise_component = component.clone();
            noise_component.arguments.insert("sensitivity".to_string(), id_sensitivity);
            graph_expansion.insert(component_id, noise_component);
        }

        Ok((current_id, graph_expansion))
    }
}
