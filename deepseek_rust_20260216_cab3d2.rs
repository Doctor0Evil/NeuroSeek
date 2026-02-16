// No soil chemistry here. No material thickness. Just structure and rules.

#[derive(NanopolyObject)]
struct EcoMat {
    id: HexStamp,
    state: EcoMatState,
    metrics: ResponseMetrics,
    soil_safety: SoilSafetyFlags,
}

enum EcoMatState {
    DeployedSensor {
        deployment_time: Timestamp,
        location: GPSCoordinate,
        silk_thickness_microns: u32, // just a number, not a material discussion
    },
    SoilEnricher {
        transition_time: Timestamp,
        expected_duration_days: u16,
    },
    RetiredAsset {
        final_verification: SoilHealthProof,
    },
}

struct ResponseMetrics {
    k: f64, // knowledge density (must be >0.9)
    d: f64, // energy to planet (must be <0.2)
    dw: f64, // psychological leverage (must be <0.1)
}

struct SoilSafetyFlags {
    roh_soil_passed: bool,          // must be true
    eco_impact_monotone: bool,      // must be true
    lifeforce_index_stable: bool,   // must be true
}

// State transition rules
impl EcoMat {
    fn evolve_to_enricher(&mut self) -> Result<(), EvolutionError> {
        require!(self.state == EcoMatState::DeployedSensor);
        require!(self.deployment_age() > MIN_SENSOR_LIFE_DAYS);
        require!(self.soil_safety.roh_soil_passed);
        
        self.state = EcoMatState::SoilEnricher {
            transition_time: now(),
            expected_duration_days: 90,
        };
        
        emit_event(EcoMatEvent::TransitionToEnricher {
            id: self.id,
            time: now(),
        });
        
        Ok(())
    }
    
    fn verify_retirement(&mut self, soil_proof: SoilHealthProof) -> Result<(), VerificationError> {
        require!(self.state == EcoMatState::SoilEnricher);
        require!(soil_proof.eco_impact_score < 0.1);
        require!(soil_proof.lifeforce_index_delta > 0.15);
        
        self.state = EcoMatState::RetiredAsset {
            final_verification: soil_proof,
        };
        
        // Mint eco-credit to deploying CITIZEN
        mint_eco_credit(soil_proof.deployer, soil_proof.eco_gain());
        
        Ok(())
    }
}