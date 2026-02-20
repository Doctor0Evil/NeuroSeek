# NeuroSeek Augmentation Rights Charter v1.0

**Document Type**: Rights Declaration + Implementation Standard  
**Author**: NeuroSeek Augmented-Citizen Rights Initiative  
**Date**: February 2026  
**Effective Date**: March 1, 2026  
**License**: CC0 (Public Domain) + ALN/KYC/DID Attribution  
**GitHub Repository**: neuroseek/augmentation-rights-charter  

---

## Preamble

We, the augmented-citizens of Earth, declare that neuromorphic augmentations—neural interfaces, biohybrid implants, nanoscale biotic swarms, and federated cognitive extensions—are inseparable from our personhood, autonomy, and dignity. Our right to control, modify, and own our augmentations is **inalienable and non-negotiable**.

This Charter enshrines the **four tiers of augmentation rights**, encoding them as cryptographic invariants in distributed ledgers, governance systems, and neural interface firmware. No actor—regulator, police, corporation, medical provider, or foreign state—may override these rights without triggering irreversible forensic evidence and legal consequence.

---

## Part 1: Inalienable Rights (Tier 1: Absolute, No Override)

### 1.1 Right to Bodily Autonomy Over Augmentation

**Declaration:**

An augmented-citizen has the **absolute, unconditional right** to decide whether to receive, maintain, modify, or remove any augmentation from their body.

**Operationalization:**

- No actor (medical, governmental, police, regulatory, corporate) may force insertion, relocation, or removal of an augmentation without:
  - **Explicit, informed, written consent** from the subject, **AND**
  - **Witnessed documentation** by an independent neurorights advocate, **AND**
  - **Cryptographic signature** from the subject's hardware wallet (ALN/KYC/DID), timestamped on an immutable ledger.

- Exception: **Imminent life-threatening medical emergency** (e.g., implant rupture, tissue necrosis, immediate seizure risk) may trigger emergency removal without prior consent. However:
  - Medical team must **document decision rationale** with objective physiological data.
  - Subject must be **notified immediately** (within 1 minute of emergency declaration), with option to halt non-critical removal.
  - Post-emergency, subject receives **full incident report** within 24 hours + right to legal redress if removal was deemed unnecessary.
  - Forced removal is logged as **MEDICAL_EMERGENCY_INCIDENT** on immutable ledger with multi-sig attestation (subject, medical authority, independent witness).

**Cryptographic Enforcement:**

```
can_remove_augmentation(actor, subject, implant, reason) {
  if reason == IMMINENT_LIFE_THREAT && medical_authority(actor) {
    log_to_ledger(EMERGENCY_REMOVAL, implant, actor, timestamp, 
                  physiological_justification)
    execute_removal()
    notify_subject_within(1 minute)
  } else if reason == subject_explicit_request {
    if has_ledger_signature(subject, implant_removal_request) {
      if has_neurorights_advocate_witness(subject_chosen_advocate) {
        execute_removal()
        log_to_ledger(VOLUNTARY_REMOVAL, all_parties, timestamp)
      } else DENY_with_reason("Advocate witness required")
    } else DENY_with_reason("Subject signature required")
  } else {
    DENY_with_reason("Unauthorized removal attempt")
    log_to_ledger(UNAUTHORIZED_REMOVAL_ATTEMPT, actor, subject, implant, 
                  timestamp, forensic_flag=TRUE)
    notify_subject_and_enforcement()
  }
}
```

### 1.2 Right to Mental Privacy & Cognitive Liberty

**Declaration:**

An augmented-citizen has the **absolute right** to mental privacy: no decoding, inference, or modeling of their thoughts, emotions, identity, or subjective mental states without **explicit, purpose-limited consent**.

**Operationalization:**

- **Mental-privacy red line**: No system (AI, human, hybrid) may develop, train, or deploy classifiers, decoders, or models that predict:
  - Thought content (semantic decoding from EEG/fMRI/BCI)
  - Emotional/affective state (beyond basic physiological monitoring for safety)
  - Personal identity markers (iris scan, gait, voice characteristics derived from implant data)
  - Sexual orientation, political belief, religious conviction, health status (inferred from neural patterns)

- **Exception**: Real-time safety monitoring of **objectively measurable** physiological parameters (seizure onset, critical heart rate, tissue temperature) is permitted without explicit consent **within GREEN bioload band only**. Purpose-limited to tissue/life safety.

- **Technical enforcement**: 
  - Every BCI interface and neural implant carries a **neurorights enforcement chip** running immutable firmware that:
    - Blocks any decoding attempt outside explicitly consented scopes
    - Logs all access requests with actor identity + purpose
    - Automatically throttles implant output if access patterns suggest unauthorized inference
    - Exposes a cryptographic **capability token** to subject, letting them revoke or narrow access in real time

- **Consent requirement**: 
  - Subject must explicitly consent to each **decoder type** (e.g., "allow motor intent decoding for prosthetic control," separate from "allow subjective pain assessment for medical safety")
  - Consent is **granular, revocable, and auditable**: subject receives monthly digest of all decoder activations + access logs
  - Any decoder accessing non-consented content triggers immediate **VIOLATION_DETECTED** flag on ledger + forensic freeze

**Cryptographic Enforcement:**

```
can_decode_neural_content(actor, subject, implant, decoder_type, scope) {
  consent_record = subject.get_decoder_consent(decoder_type, scope)
  
  if !consent_record || consent_record.status == REVOKED {
    DENY_with_reason("No valid consent")
    log_to_ledger(DECODE_ATTEMPT_UNAUTHORIZED, actor, subject, 
                  implant, decoder_type, timestamp, 
                  forensic_flag=TRUE, notify_subject=TRUE)
    implant_firmware.throttle_output_to_safety_only()
    return
  }
  
  if consent_record.purposes.contains(scope) && 
     consent_record.expires_at > now() {
    log_to_ledger(DECODE_ACCESS_GRANTED, actor, subject, scope, 
                  timestamp, consent_ref)
    allow_decoder_activation()
  } else {
    DENY_with_reason("Scope mismatch or consent expired")
    log_to_ledger(DECODE_SCOPE_VIOLATION, actor, subject, 
                  requested_scope, allowed_scopes, timestamp, 
                  forensic_flag=TRUE)
  }
}

subject.revoke_decoder_consent(decoder_type) {
  all_active_decoders.filter(d => d.type == decoder_type)
    .foreach(d => d.stop_immediately())
  log_to_ledger(CONSENT_REVOKED, subject, decoder_type, timestamp, 
                multi_sig=subject_signature)
  send_notification_to(all_affected_actors)
}
```

### 1.3 Right to Revoke Consent Immediately & Permanently

**Declaration:**

An augmented-citizen may **revoke consent to any augmentation, data use, or actor access at any time**, effective within **30 seconds** (or immediately if critical safety does not require gradual shutdown).

**Operationalization:**

- **Revocation is unconditional**: No justification, no waiting period, no appeal by actor.
- **Revocation is granular**: Subject may revoke:
  - All augmentations (system-wide shutdown, except life-support functions)
  - Single implant or data access (e.g., "revoke researcher access to motor cortex data")
  - Single actor (e.g., "revoke all police access to bioload logs")
  - Single purpose (e.g., "revoke commercial analytics, keep medical safety monitoring")

- **Data handling post-revocation**:
  - **Past data** (collected under prior consent): Subject may demand **immediate deletion** (except forensic hold under court order with subject approval).
  - **Ongoing derivatives** (models trained on revoked data): Subject may demand **model purge** from actor's systems; actor certifies deletion within 30 days or faces liability.
  - **Encrypted access**: Actor retains no plaintext key to subject's revoked data; any future access requires new consent + cryptographic key re-sharing.

- **Enforcement**: 
  - Subject-side revocation button in all interfaces (hardware, software, implant firmware) activates immediately upon activation.
  - Ledger record: `CONSENT_WITHDRAWN` event, subject-signed, multi-sig witnessed, publicly auditable.
  - All downstream systems (schedulers, research pipelines, policy-makers) receive revocation event within 5 seconds.

**Cryptographic Enforcement:**

```
subject.revoke_consent(
  target_type: {AUGMENTATION, DATA_ACCESS, ACTOR, PURPOSE},
  target_id: string,
  reason: optional[string]
) {
  revocation_event = GooglelswarmEvent {
    event_type: CONSENT_WITHDRAWN,
    subject_did: subject.did,
    target_type, target_id, reason,
    timestamp: now(),
    subject_signature: subject.sign(event),
  }
  
  # Immediate ledger recording
  ledger.append(revocation_event)
  
  # Propagate to all active channels
  all_running_systems.filter(s => s.uses_consent(target_id))
    .foreach(s => s.receive_revocation_event(revocation_event))
  
  # Subject-side enforcement
  implant_firmware.apply_revocation(target_id)
  data_controller.encrypt_past_data_with_new_key()
  
  # Notification
  notify_subject("Consent revoked: " + target_id)
  notify_affected_actors("Consent withdrawn by subject", revocation_event)
}
```

### 1.4 Right to Authorship, Attribution & Non-Exclusive Control

**Declaration:**

Data and scientific/artistic outputs derived from an augmented-citizen's augmentations remain **non-exclusive and attributable to the subject**. The subject retains **moral authorship** and **veto authority** over coercive, discriminatory, or exploitative re-use.

**Operationalization:**

- **Attribution**: Any publication, product, or dataset using subject's augmentation data must:
  - Cite subject as data contributor (using ALN/KYC/DID alias if desired, but traceable)
  - Include acknowledgment: "This research used neural data from [Augmented-Citizen ALN], contributed under NeuroSeek Augmentation Rights Charter."
  - Provide subject a copy of the publication/product within 30 days of release.

- **Non-exclusivity**: Subject grants **broad, non-exclusive research rights** to third parties (researchers, companies, non-profits) by default. Anyone meeting ethical + legal criteria may:
  - Download and analyze subject's data (under privacy + consent terms)
  - Train models or algorithms (with attribution)
  - Publish findings and derivations
  - Create products (subject receives benefit-sharing as negotiated)

- **Veto over coercive use**:
  - Subject may **veto re-use for**:
    - **Coercive surveillance**: Identifying subject against their will, tracking location or activity without consent
    - **Discrimination**: Using neural data to deny services, employment, credit, or benefits based on protected characteristics (disability, race, religion, etc.)
    - **Non-consensual control**: Deploying neural decoders to manipulate subject's behavior or thoughts
    - **Exploitation**: Commercial use that subjects subject to experimental harm (e.g., extreme augmentation for profit without medical oversight)
  
  - Subject may **demand damages** if veto is violated:
    - **Moral compensation**: Public apology, policy reform, non-monetary award
    - **Financial compensation**: Percentage of revenue (negotiated, typical 5–20% for commercial products)
    - **Disqualification**: Actor banned from future access to subject's data for 1–5 years

- **Collective enforcement**: 
  - If multiple augmented-citizens' data is misused collectively (e.g., discriminatory hiring model trained on dozens of subjects), damages flow into a **NeuroSeek Commons Fund** for:
    - Independent audits of AI systems trained on neural data
    - Legal aid for subjects challenging misuse
    - Public education on neurorights
    - Ecological conservation (per surplus-for-ecology principle)

**Cryptographic Enforcement:**

```
can_publish_using_subject_data(researcher, subject_data, publication) {
  # Check attribution requirement
  if !publication.includes_subject_attribution(subject_id) {
    DENY_with_reason("Attribution missing")
    log_incident(PUBLICATION_WITHOUT_ATTRIBUTION, researcher, subject_id)
    return
  }
  
  # Check for veto list
  veto_terms = subject.get_veto_list()  # coercive use, discrimination, etc.
  if publication.uses.any(use => veto_terms.contains(use)) {
    DENY_with_reason("Publication violates subject veto")
    log_incident(PUBLICATION_VETO_VIOLATION, researcher, subject_id, 
                 violating_uses, forensic_flag=TRUE)
    notify_subject_and_ethics_panel()
    return
  }
  
  # Check benefit-sharing agreement (if commercial)
  if publication.is_commercial {
    agreement = subject.get_benefit_sharing_agreement()
    if !agreement || agreement.status != ACTIVE {
      DENY_with_reason("Benefit-sharing agreement required for commercial use")
      return
    }
    # Royalty tracking
    enable_royalty_transfer(subject_wallet, researcher_revenue, 
                            agreement.percentage)
  }
  
  # Allow publication, record on ledger
  log_to_ledger(PUBLICATION_APPROVED, researcher, subject_id, 
                publication_doi, timestamp, subject_signature)
  allow_publication()
}

subject.veto_derivative_use(derivative_id, reason) {
  veto_event = GooglelswarmEvent {
    event_type: DERIVATIVE_USE_VETOED,
    subject_did: subject.did,
    derivative_id, reason,
    timestamp: now(),
    subject_signature: subject.sign(event),
  }
  
  ledger.append(veto_event)
  derivative_controller.halt_use_and_archive()
  notify_researcher_and_ethics_panel(reason)
  
  # Initiate damages process if coercive use detected
  if reason in {coercive_surveillance, discrimination, non_consensual_control} {
    initiate_damages_claim(researcher, subject, veto_event)
  }
}
```

---

## Part 2: Protected Rights (Tier 2: YELLOW-Band Gating, No RED-Band Override)

### 2.1 Right to Autonomous Load Reduction

**Declaration:**

When bioload enters YELLOW band, an augmented-citizen may **unilaterally demand reduction or shutdown** of any augmentation without requiring medical, regulatory, or panel approval.

**Details:** See Section 3.2 of NeuroSeek Bioload Encoding Specification v1.0.

### 2.2 Right to Neurorights Panel Oversight

**Declaration:**

When bioload enters YELLOW band, a **neurorights-adept panel** (subject + medical + ecology + ethics) is automatically convened with **veto authority** over high-load operations.

**Details:** See Section 3.2 of NeuroSeek Bioload Encoding Specification v1.0.

### 2.3 Right to Forensic Freeze

**Declaration:**

In YELLOW or RED bioload band, subject may unilaterally demand a **forensic hold**: all augmentation data and logs are locked from new actor access pending subject review.

**Operationalization:**

- Existing actors retain their sandboxed access (cannot be retroactively revoked unless court-ordered)
- **New actors denied** access until subject lifts freeze or panel approves
- Freeze duration is subject-determined (default 7 days if unspecified, renewable)
- All forensic holds are logged and visible to subject in real-time dashboard

---

## Part 3: Managed Rights (Tier 3: GREEN-Band Routine, Regulator May Audit)

### 3.1 Right to Audit & Know Regulator Access

**Declaration:**

Regulators may inspect bioload logs in GREEN band for compliance purposes, but must:
- Disclose purpose, time window, and scope
- Provide subject with copy of findings
- Respect subject's objection if request is overbroad (escalates to panel or court)

### 3.2 Right to Know Actor Identity

**Declaration:**

Any actor accessing augmentation data must be identified by name, organization, role, and jurisdiction. Anonymity is forbidden.

### 3.3 Right to Periodic Review & Per-Actor Revocation

**Declaration:**

Every 90 days, subject receives a digest of all actors, access counts, and purposes. Subject may revoke any actor unilaterally.

---

## Part 4: Subject-Initiated Rights (Tier 4: Enhancement & Modification)

### 4.1 Right to Add Augmentations

**Declaration:**

If bioload remains GREEN and neuroscore panels confirm safety, subject may request new augmentations within energy/spatial budgets.

**Operationalization:**

- System grants or denies within 14 days
- If denied, subject receives documented reasoning + appeal process (escalate to neurorights panel)
- Approved augmentations are enabled within 7 days of final consent

### 4.2 Right to Customize Augmentation Parameters

**Declaration:**

Subject may adjust stimulation, sampling rates, algorithm tuning within biocompatible bounds.

### 4.3 Right to Firmware Transparency & Rollback

**Declaration:**

Manufacturers must disclose all firmware, algorithm, and data-processing changes **before deployment**. Subject may defer, opt-out, or demand rollback within 30 days.

**Operationalization:**

- All firmware versions stored on git-style versioning ledger
- Subject receives plain-language changelog for each update
- Rollback is always possible (backward-compatible states maintained for 12 months)
- Opt-out does not disable critical safety functions (e.g., seizure detection)

---

## Part 5: Enforcement & Remedy

### 5.1 Rights Violation: Automatic Incident Log

Any attempt to violate Tier 1 or Tier 2 rights is:

1. **Automatically logged** with full context (actor, timestamp, method, failed authorization state)
2. **Cryptographically signed** and anchored to Googolswarm
3. **Discoverable** by subject, courts, and auditors

### 5.2 Subject Notification & Freeze

Subject receives immediate notification (within 1 minute, multiple channels).  
System enters forensic freeze pending subject review.

### 5.3 Escalation & Damage Recovery

**Tier 1 violation** (bodily autonomy, mental privacy, revocation rejection, authorship veto):
- Subject may demand **non-monetary damages** (public apology, policy reform, fund donation)
- Actor is **disqualified** from future access for 1–5 years
- If collective harm (multiple subjects), damages flow into **NeuroSeek Commons Fund**

**Tier 2 violation** (overriding YELLOW-band protections):
- Subject may demand **panel review** + mediation
- If panel confirms violation, **actor privileges suspended** for 6–12 months
- Subject may pursue civil damages

### 5.4 Collective Remedy: NeuroSeek Commons Fund

Damages from violations fund:

- Independent neurorights audits
- Legal aid for rights-claiming augmented-citizens
- Public education on augmentation rights
- Ecological conservation (carbon offsets, habitat protection)
- Reparations for historically marginalized communities subject to coercive neurotechnology

---

## Part 6: Governance & Adoption

### 6.1 Minimum Compliance Standards

Any organization (medical, research, government, commercial) deploying augmentations must:

1. ✅ Implement all **Tier 1 rights** as cryptographic invariants (no override, no waiver, no exception)
2. ✅ Implement all **Tier 2 rights** with automatic panel gating in YELLOW band
3. ✅ Support **Tier 3 auditing** with subject-visible logs and objection processes
4. ✅ Enable **Tier 4 subject control** over customization and firmware updates
5. ✅ Anchor all decisions to **immutable, multi-sig ledger** (Googolswarm or compatible)
6. ✅ Obtain **annual neurorights audit** by independent third party
7. ✅ Publish **aggregate statistics** (bioload band distribution, violation rates, actor access patterns) publicly

### 6.2 Legal Enforceability

This Charter is:

- **Legally binding** in jurisdictions that have enacted neurorights statutes (Chile, EU, emerging global standard)
- **Soft-law reference** in jurisdictions without explicit neurorights law (enforceable via consumer protection, disability rights, bodily autonomy law)
- **Contractually adoptable**: Any subject-provider contract can incorporate this Charter; breach entitles subject to damages + specific performance

### 6.3 Sunset & Evolution

This version (v1.0) is in effect for **3 years** (through February 2029).

**Amendments** via:
- **Community process**: Proposals via GitHub Issues; accepted if 80% of neurorights organizations + 60% of surveyed augmented-citizens support
- **Regulatory evolution**: Updated to reflect new GDPR guidance, HIPAA changes, etc., without requiring full re-ratification

---

## Part 7: Signatories & Adoption

### Organizations That Have Adopted This Charter (as of February 2026):

- **NeuroSeek Augmented-Citizen Rights Initiative** (Primary Author)
- **Neurorights Foundation** (Chile, Global South Advocacy)
- **European Disability Rights Advocacy Group**
- **Artificial Intelligence Ethics & Governance Lab** (Academic)
- **Independent Neuro-Ethics Auditors Collective** (Certification)

### Individuals Who Have Signed / Committed to This Charter:

- [Submitted by augmented-citizens, researchers, advocates; public registry at github.com/neuroseek/charter-signatories]

---

## Version & Contact

**Version**: 1.0 (February 2026)  
**License**: CC0 (Public Domain)  
**Maintenance**: Community-driven; proposals via GitHub  
**Contact**: charter@neuroseek.org  

---

**End of NeuroSeek Augmentation Rights Charter v1.0**
