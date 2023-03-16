use rand::distributions::Distribution;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use statrs::distribution::Laplace;
use std::collections::HashMap;
use tracing::{debug, info, warn};


#[derive(Debug, Deserialize, Serialize)]
struct Period {
    end: String,
    start: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ValueQuantity {
    code: String,
    system: String,
    unit: String,
    value: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Extension {
    url: String,
    value_quantity: ValueQuantity,
}

#[derive(Debug, Deserialize, Serialize)]
struct Code {
    text: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Coding {
    code: String,
    system: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Population {
    code: Code,
    count: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Group {
    code: Code,
    population: Value,
    stratifier: Value,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MeasureReport {
    date: String,
    extension: Vec<Extension>,
    group: Vec<Group>,
    measure: String,
    period: Period,
    resource_type: String,
    status: String,
    type_: String, //because "type" is a reserved keyword
}

// obfuscation cache
type Sensitivity = usize;
type Count = u64;
type Bin = usize;
pub struct ObfCache {
    pub cache: HashMap<(Sensitivity, Count, Bin), u64>,
}

const DELTA_PATIENT: f64 = 1.;
const DELTA_SPECIMEN: f64 = 20.;
const DELTA_DIAGNOSIS: f64 = 3.;
const EPSILON: f64 = 0.1;


pub fn obfuscate_counts(json_str: &str, obf_cache: &mut ObfCache) -> String {

    let patient_dist = Laplace::new(0.0, DELTA_PATIENT/EPSILON).unwrap(); //TODO error handling
    let diagnosis_dist = Laplace::new(0.0, DELTA_DIAGNOSIS/EPSILON).unwrap();
    let specimen_dist = Laplace::new(0.0, DELTA_SPECIMEN/EPSILON).unwrap();

    let mut measure_report: MeasureReport = serde_json::from_str(&json_str).unwrap();
    for g in &mut measure_report.group {
        match &g.code.text[..] {
            "patients" => {
                info!("patients");
                obfuscate_counts_recursive(&mut g.population, &patient_dist, 1, obf_cache);
                obfuscate_counts_recursive(&mut g.stratifier, &patient_dist, 2, obf_cache);
            }
            "diagnosis" => {
                info!("diagnosis");
                obfuscate_counts_recursive(&mut g.population, &diagnosis_dist, 1, obf_cache);
                obfuscate_counts_recursive(&mut g.stratifier, &diagnosis_dist, 2, obf_cache);
            }
            "specimen" => {
                info!("specimen");
                obfuscate_counts_recursive(&mut g.population, &specimen_dist, 1, obf_cache);
                obfuscate_counts_recursive(&mut g.stratifier, &specimen_dist, 2, obf_cache);
            }
            _ => {
                warn!("focus is not aware of this type of stratifier")
            }
        }
    }

    let measure_report_obfuscated = serde_json::to_string_pretty(&measure_report).unwrap(); //TODO error handling
    dbg!(measure_report_obfuscated.clone());
    measure_report_obfuscated
}

fn obfuscate_counts_recursive(val: &mut Value, dist: &Laplace, bin: Bin, obf_cache: &mut ObfCache) {
    match val {
        Value::Object(map) => {
            if let Some(count_val) = map.get_mut("count") {
                if let Some(count) = count_val.as_u64() {
                    if count >= 1 && count <= 10 {
                        *count_val = json!(10);
                    } else if count > 10 {
                        let mut rng = thread_rng();
                        let sensitivity: usize = (dist.scale() * EPSILON).round() as usize;
                        
                        let perturbation = match obf_cache.cache.get(&(sensitivity, count, bin)) {
                            Some(perturbation_reference) => *perturbation_reference,
                            None => {
                                let perturbation_value = dist.sample(&mut rng).round() as u64;
                                obf_cache.cache.insert((sensitivity, count, bin), perturbation_value);
                                perturbation_value
                            }
                        };

                        *count_val = json!((count + perturbation + 5) / 10 * 10);
                        // Per data protection concept it must be rounded to the nearest multiple of 10
                        // "Counts of patients and samples undergo obfuscation on site before being sent to central infrastructure. This is done by incorporating some randomness into the count and then rounding it to the nearest multiple of ten."
                    } // And zero stays zero
                }
            }
            for (_, sub_val) in map.iter_mut() {
                obfuscate_counts_recursive(sub_val, dist, bin, obf_cache);
            }
        }
        Value::Array(vec) => {
            for sub_val in vec.iter_mut() {
                obfuscate_counts_recursive(sub_val, dist, bin, obf_cache);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_JSON: &str = r#"
    {
        "date": "2023-03-03T15:54:21.740195064Z",
        "extension": [
            {
                "url": "https://samply.github.io/blaze/fhir/StructureDefinition/eval-duration",
                "valueQuantity": {
                    "code": "s",
                    "system": "http://unitsofmeasure.org",
                    "unit": "s",
                    "value": 0.050495759
                }
            }
        ],
        "group": [
            {
                "code": {
                    "text": "patients"
                },
                "population": [
                    {
                        "code": {
                            "coding": [
                                {
                                    "code": "initial-population",
                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                }
                            ]
                        },
                        "count": 74
                    }
                ],
                "stratifier": [
                    {
                        "code": [
                            {
                                "text": "Gender"
                            }
                        ],
                        "stratum": [
                            {
                                "population": [
                                    {
                                        "code": {
                                            "coding": [
                                                {
                                                    "code": "initial-population",
                                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                                }
                                            ]
                                        },
                                        "count": 31
                                    }
                                ],
                                "value": {
                                    "text": "female"
                                }
                            },
                            {
                                "population": [
                                    {
                                        "code": {
                                            "coding": [
                                                {
                                                    "code": "initial-population",
                                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                                }
                                            ]
                                        },
                                        "count": 43
                                    }
                                ],
                                "value": {
                                    "text": "male"
                                }
                            }
                        ]
                    },
                    {
                        "code": [
                            {
                                "text": "Age"
                            }
                        ],
                        "stratum": [
                            {
                                "population": [
                                    {
                                        "code": {
                                            "coding": [
                                                {
                                                    "code": "initial-population",
                                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                                }
                                            ]
                                        },
                                        "count": 5
                                    }
                                ],
                                "value": {
                                    "text": "40"
                                }
                            },
                            {
                                "population": [
                                    {
                                        "code": {
                                            "coding": [
                                                {
                                                    "code": "initial-population",
                                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                                }
                                            ]
                                        },
                                        "count": 4
                                    }
                                ],
                                "value": {
                                    "text": "50"
                                }
                            },
                            {
                                "population": [
                                    {
                                        "code": {
                                            "coding": [
                                                {
                                                    "code": "initial-population",
                                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                                }
                                            ]
                                        },
                                        "count": 14
                                    }
                                ],
                                "value": {
                                    "text": "60"
                                }
                            },
                            {
                                "population": [
                                    {
                                        "code": {
                                            "coding": [
                                                {
                                                    "code": "initial-population",
                                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                                }
                                            ]
                                        },
                                        "count": 4
                                    }
                                ],
                                "value": {
                                    "text": "80"
                                }
                            }
                        ]
                    },
                    {
                        "code": [
                            {
                                "text": "Custodian"
                            }
                        ],
                        "stratum": [
                            {
                                "population": [
                                    {
                                        "code": {
                                            "coding": [
                                                {
                                                    "code": "initial-population",
                                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                                }
                                            ]
                                        },
                                        "count": 31
                                    }
                                ],
                                "value": {
                                    "text": "bbmri-eric:ID:CZ_CUNI_PILS:collection:serum_plasma"
                                }
                            },
                            {
                                "population": [
                                    {
                                        "code": {
                                            "coding": [
                                                {
                                                    "code": "initial-population",
                                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                                }
                                            ]
                                        },
                                        "count": 43
                                    }
                                ],
                                "value": {
                                    "text": "null"
                                }
                            }
                        ]
                    }
                ]
            },
            {
                "code": {
                    "text": "diagnosis"
                },
                "population": [
                    {
                        "code": {
                            "coding": [
                                {
                                    "code": "initial-population",
                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                }
                            ]
                        },
                        "count": 324
                    }
                ],
                "stratifier": [
                    {
                        "code": [
                            {
                                "text": "diagnosis"
                            }
                        ],
                        "stratum": [
                            {
                                "population": [
                                    {
                                        "code": {
                                            "coding": [
                                                {
                                                    "code": "initial-population",
                                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                                }
                                            ]
                                        },
                                        "count": 26
                                    }
                                ],
                                "value": {
                                    "text": "C34.0"
                                }
                            },
                            {
                                "population": [
                                    {
                                        "code": {
                                            "coding": [
                                                {
                                                    "code": "initial-population",
                                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                                }
                                            ]
                                        },
                                        "count": 28
                                    }
                                ],
                                "value": {
                                    "text": "C34.2"
                                }
                            },
                            {
                                "population": [
                                    {
                                        "code": {
                                            "coding": [
                                                {
                                                    "code": "initial-population",
                                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                                }
                                            ]
                                        },
                                        "count": 25
                                    }
                                ],
                                "value": {
                                    "text": "C34.8"
                                }
                            },
                            {
                                "population": [
                                    {
                                        "code": {
                                            "coding": [
                                                {
                                                    "code": "initial-population",
                                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                                }
                                            ]
                                        },
                                        "count": 27
                                    }
                                ],
                                "value": {
                                    "text": "C78.0"
                                }
                            },
                            {
                                "population": [
                                    {
                                        "code": {
                                            "coding": [
                                                {
                                                    "code": "initial-population",
                                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                                }
                                            ]
                                        },
                                        "count": 25
                                    }
                                ],
                                "value": {
                                    "text": "D38.6"
                                }
                            },
                            {
                                "population": [
                                    {
                                        "code": {
                                            "coding": [
                                                {
                                                    "code": "initial-population",
                                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                                }
                                            ]
                                        },
                                        "count": 25
                                    }
                                ],
                                "value": {
                                    "text": "R91"
                                }
                            }
                        ]
                    }
                ]
            },
            {
                "code": {
                    "text": "specimen"
                },
                "population": [
                    {
                        "code": {
                            "coding": [
                                {
                                    "code": "initial-population",
                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                }
                            ]
                        },
                        "count": 124
                    }
                ],
                "stratifier": [
                    {
                        "code": [
                            {
                                "text": "sample_kind"
                            }
                        ],
                        "stratum": [
                            {
                                "population": [
                                    {
                                        "code": {
                                            "coding": [
                                                {
                                                    "code": "initial-population",
                                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                                }
                                            ]
                                        },
                                        "count": 62
                                    }
                                ],
                                "value": {
                                    "text": "blood-plasma"
                                }
                            },
                            {
                                "population": [
                                    {
                                        "code": {
                                            "coding": [
                                                {
                                                    "code": "initial-population",
                                                    "system": "http://terminology.hl7.org/CodeSystem/measure-population"
                                                }
                                            ]
                                        },
                                        "count": 62
                                    }
                                ],
                                "value": {
                                    "text": "blood-serum"
                                }
                            }
                        ]
                    }
                ]
            }
        ],
        "measure": "urn:uuid:fe7e5bf7-d792-4368-b1d2-5798930db13e",
        "period": {
            "end": "2030",
            "start": "2000"
        },
        "resourceType": "MeasureReport",
        "status": "complete",
        "type": "summary"
    }
    "#;


    #[test]
    fn test_obfuscate_counts() {
        let mut obf_cache = ObfCache { cache: HashMap::new() };
        let obfuscated_json = obfuscate_counts(EXAMPLE_JSON, &mut obf_cache);

        // Check that the obfuscated JSON can be parsed and has the same structure as the original JSON
        let _: MeasureReport = serde_json::from_str(&obfuscated_json).unwrap();

        // Check that the obfuscated JSON is different from the original JSON
        assert_ne!(obfuscated_json, EXAMPLE_JSON);

        // Check that obfuscating the same JSON twice with the same obfuscation cache gives the same result
        let obfuscated_json_2 = obfuscate_counts(EXAMPLE_JSON, &mut obf_cache);
        assert_eq!(obfuscated_json, obfuscated_json_2);
    }

}
