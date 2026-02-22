package com.neuroseek.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.LockOpen
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.neuroseek.governance.*

/**
 * NeuroConsentGateScreen: Main UI for consent ledger and authorization decisions
 */
@Composable
fun NeuroConsentGateScreen(
    viewModel: NeuroConsentViewModel = viewModel()
) {
    val uiState by viewModel.uiState.collectAsState()
    val bioloadBand by viewModel.currentBioloadBand.collectAsState()
    
    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Color(0xFFF5F5F5))
            .verticalScroll(rememberScrollState())
    ) {
        // Header
        TopAppBar(
            title = { Text("NeuroSeek Consent Ledger", fontWeight = FontWeight.Bold) },
            colors = TopAppBarDefaults.topAppBarColors(
                containerColor = Color(0xFF1A1A2E)
            )
        )
        
        // Current Bioload Band
        BioloadBandIndicator(band = bioloadBand)
        
        // Subject Identity Card
        SubjectIdentityCard(
            subject = uiState.currentSubject,
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)
        )
        
        // Current Session (SpectralConference)
        CurrentSessionPanel(
            session = uiState.activeSession,
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)
        )
        
        // Pending Authorization Requests
        PendingAuthorizationPanel(
            requests = uiState.pendingRequests,
            onApprove = { viewModel.approveRequest(it) },
            onDeny = { viewModel.denyRequest(it) },
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)
        )
        
        // Recent Ledger Events
        RecentEventsPanel(
            events = uiState.recentEvents,
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)
        )
        
        // Augmentation Rights Display
        AugmentationRightsPanel(
            rights = uiState.rights,
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)
        )
        
        Spacer(modifier = Modifier.height(24.dp))
    }
}

@Composable
fun SubjectIdentityCard(
    subject: NeuroSubjectId?,
    modifier: Modifier = Modifier
) {
    if (subject == null) return
    
    Card(
        modifier = modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        colors = CardDefaults.cardColors(containerColor = Color.White),
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            Text(
                text = "Subject Identity",
                fontWeight = FontWeight.Bold,
                fontSize = 14.sp,
                modifier = Modifier.padding(bottom = 8.dp)
            )
            
            IdentityRow("ALN Address", subject.alnAddress)
            IdentityRow("DID URI", subject.didUri)
            IdentityRow("KYC Verified", if (subject.kycVerified) "✓ Yes" else "✗ No")
            IdentityRow("Profile", subject.biostretched_zone_profile)
        }
    }
}

@Composable
private fun IdentityRow(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(text = label, fontSize = 12.sp, color = Color.Gray)
        Text(
            text = value,
            fontSize = 12.sp,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.widthIn(max = 150.dp)
        )
    }
}

@Composable
fun CurrentSessionPanel(
    session: SpectralConferenceSession?,
    modifier: Modifier = Modifier
) {
    if (session == null) return
    
    Card(
        modifier = modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = Color(0xFFE3F2FD))
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            Text(
                text = "Active SpectralConference",
                fontWeight = FontWeight.Bold,
                fontSize = 14.sp
            )
            
            Spacer(modifier = Modifier.height(8.dp))
            
            SessionInfoRow("Session ID", session.regionSessionKey)
            SessionInfoRow("Zone", session.zone)
            SessionInfoRow("Participants", session.participantCount.toString())
            SessionInfoRow("Duration", "${session.elapsedSeconds}s active")
        }
    }
}

@Composable
private fun SessionInfoRow(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 3.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(text = label, fontSize = 11.sp, color = Color.Gray)
        Text(text = value, fontSize = 11.sp, fontWeight = FontWeight.Bold)
    }
}

@Composable
fun PendingAuthorizationPanel(
    requests: List<AuthorizationRequest>,
    onApprove: (String) -> Unit,
    onDeny: (String) -> Unit,
    modifier: Modifier = Modifier
) {
    Card(
        modifier = modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = Color.White)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            Text(
                text = "Pending Authorizations (${requests.size})",
                fontWeight = FontWeight.Bold,
                fontSize = 14.sp,
                modifier = Modifier.padding(bottom = 12.dp)
            )
            
            if (requests.isEmpty()) {
                Text(
                    text = "No pending requests",
                    fontSize = 12.sp,
                    color = Color.Gray,
                    modifier = Modifier.padding(vertical = 8.dp)
                )
            } else {
                requests.forEach { request ->
                    AuthorizationRequestCard(
                        request = request,
                        onApprove = { onApprove(request.requestId) },
                        onDeny = { onDeny(request.requestId) },
                        modifier = Modifier.padding(bottom = 8.dp)
                    )
                }
            }
        }
    }
}

@Composable
fun AuthorizationRequestCard(
    request: AuthorizationRequest,
    onApprove: () -> Unit,
    onDeny: () -> Unit,
    modifier: Modifier = Modifier
) {
    Card(
        modifier = modifier
            .fillMaxWidth()
            .background(Color(0xFFFFF9C4)),
        colors = CardDefaults.cardColors(containerColor = Color(0xFFFFF9C4))
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp)
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 8.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = request.actorOrganization,
                        fontWeight = FontWeight.Bold,
                        fontSize = 12.sp
                    )
                    Text(
                        text = request.purpose,
                        fontSize = 10.sp,
                        color = Color.Gray
                    )
                }
                
                Icon(
                    imageVector = if (request.decision == AuthorizationDecision.PENDING) Icons.Default.Lock else Icons.Default.LockOpen,
                    contentDescription = "Lock status",
                    tint = if (request.decision == AuthorizationDecision.PENDING) Color(0xFFFBC02D) else Color(0xFF2E7D32)
                )
            }
            
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Button(
                    onClick = onApprove,
                    modifier = Modifier
                        .weight(1f)
                        .height(32.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = Color(0xFF2E7D32))
                ) {
                    Text("Approve", fontSize = 10.sp)
                }
                
                Button(
                    onClick = onDeny,
                    modifier = Modifier
                        .weight(1f)
                        .height(32.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = Color(0xFFC62828))
                ) {
                    Text("Deny", fontSize = 10.sp)
                }
            }
        }
    }
}

@Composable
fun RecentEventsPanel(
    events: List<LedgerEvent>,
    modifier: Modifier = Modifier
) {
    Card(
        modifier = modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = Color.White)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            Text(
                text = "Recent Ledger Events",
                fontWeight = FontWeight.Bold,
                fontSize = 14.sp,
                modifier = Modifier.padding(bottom = 12.dp)
            )
            
            events.take(5).forEach { event ->
                LedgerEventRow(event)
            }
        }
    }
}

@Composable
fun LedgerEventRow(event: LedgerEvent) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 6.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = event.event_kind.name.replace("_", " "),
                fontSize = 11.sp,
                fontWeight = FontWeight.Bold
            )
            Text(
                text = java.time.Instant.ofEpochMilli(event.timestamp)
                    .toString()
                    .take(19),
                fontSize = 9.sp,
                color = Color.Gray
            )
        }
    }
    
    Divider(modifier = Modifier.padding(vertical = 4.dp))
}

@Composable
fun AugmentationRightsPanel(
    rights: RightsSurface?,
    modifier: Modifier = Modifier
) {
    if (rights == null) return
    
    Card(
        modifier = modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = Color(0xFFF3E5F5))
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            Text(
                text = "Inalienable Rights (Tier 1)",
                fontWeight = FontWeight.Bold,
                fontSize = 12.sp,
                modifier = Modifier.padding(bottom = 8.dp)
            )
            
            RightCheckmark("Bodily Autonomy", true)
            RightCheckmark("Mental Privacy", rights.soul_modeling_forbidden)
            RightCheckmark("Consent Revocable", rights.consent_withdrawal_instantaneous)
            RightCheckmark("Authorship Protection", rights.authorship_irrevocable)
        }
    }
}

@Composable
private fun RightCheckmark(label: String, isProtected: Boolean) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(text = label, fontSize = 11.sp)
        Text(
            text = if (isProtected) "✓" else "✗",
            fontSize = 14.sp,
            color = if (isProtected) Color(0xFF2E7D32) else Color(0xFFC62828),
            fontWeight = FontWeight.Bold
        )
    }
}

// Data classes for UI state
data class NeuroConsentUiState(
    val currentSubject: NeuroSubjectId? = null,
    val activeSession: SpectralConferenceSession? = null,
    val pendingRequests: List<AuthorizationRequest> = emptyList(),
    val recentEvents: List<LedgerEvent> = emptyList(),
    val rights: RightsSurface? = null,
)

data class SpectralConferenceSession(
    val regionSessionKey: String,
    val zone: String,
    val participantCount: Int,
    val elapsedSeconds: Long,
)

data class AuthorizationRequest(
    val requestId: String,
    val actorOrganization: String,
    val purpose: String,
    val decision: String,
)

object AuthorizationDecision {
    const val PENDING = "PENDING"
    const val APPROVED = "APPROVED"
    const val DENIED = "DENIED"
}
