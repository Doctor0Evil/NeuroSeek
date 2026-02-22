package com.neuroseek.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.neuroseek.governance.*
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch

class NeuroConsentViewModel : ViewModel() {
    
    private val evaluator = NeuroConsentEvaluator()
    
    private val _uiState = MutableStateFlow(NeuroConsentUiState())
    val uiState: StateFlow<NeuroConsentUiState> = _uiState.asStateFlow()
    
    private val _currentBioloadBand = MutableStateFlow(BioloadBand.GREEN)
    val currentBioloadBand: StateFlow<BioloadBand> = _currentBioloadBand.asStateFlow()
    
    init {
        loadInitialState()
    }
    
    private fun loadInitialState() {
        viewModelScope.launch {
            // Simulate loading current subject and session
            val mockSubject = NeuroSubjectId(
                alnAddress = "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
                didUri = "did:bostrom:18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
                kycVerified = true,
                biostretched_zone_profile = "EU_GDPR_v3.1"
            )
            
            val mockSession = SpectralConferenceSession(
                regionSessionKey = "0x535045435452414C5F434F4E464552454E43455F5631",
                zone = "XRCONTROL",
                participantCount = 3,
                elapsedSeconds = 3600L
            )
            
            val mockRights = RightsSurface(
                neurorights_compliant = true,
                soul_modeling_forbidden = true,
                non_interference_required = true,
                mental_privacy_max = true,
                ecological_surplus_required = true,
                authorship_irrevocable = true,
                consent_withdrawal_instantaneous = true
            )
            
            _uiState.update { state ->
                state.copy(
                    currentSubject = mockSubject,
                    activeSession = mockSession,
                    rights = mockRights,
                    pendingRequests = listOf(
                        AuthorizationRequest(
                            requestId = "req-001",
                            actorOrganization = "Medical Research Institute",
                            purpose = "Cognitive Liberty Study",
                            decision = AuthorizationDecision.PENDING
                        )
                    )
                )
            }
        }
    }
    
    fun approveRequest(requestId: String) {
        viewModelScope.launch {
            _uiState.update { state ->
                state.copy(
                    pendingRequests = state.pendingRequests.filter { it.requestId != requestId }
                )
            }
        }
    }
    
    fun denyRequest(requestId: String) {
        viewModelScope.launch {
            _uiState.update { state ->
                state.copy(
                    pendingRequests = state.pendingRequests.filter { it.requestId != requestId }
                )
            }
        }
    }
}
