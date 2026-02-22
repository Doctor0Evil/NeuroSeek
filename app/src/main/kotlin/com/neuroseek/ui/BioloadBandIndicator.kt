package com.neuroseek.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.neuroseek.governance.BioloadBand

/**
 * BioloadBandIndicator: Visual indicator for current bioload state
 * Color-coded: GREEN (safe), YELLOW (caution), RED (emergency)
 */
@Composable
fun BioloadBandIndicator(
    band: BioloadBand,
    modifier: Modifier = Modifier,
) {
    val (backgroundColor, textColor, labelText) = when (band) {
        BioloadBand.GREEN -> Triple(
            Color(0xFF2E7D32),  // Safe green
            Color.White,
            "SAFE OPERATION"
        )
        BioloadBand.YELLOW -> Triple(
            Color(0xFFFBC02D),  // Caution yellow
            Color.Black,
            "⚠️ BIOSTRETCHED-ZONE"
        )
        BioloadBand.RED -> Triple(
            Color(0xFFC62828),  // Emergency red
            Color.White,
            "🚨 EMERGENCY RESPONSE"
        )
    }
    
    Surface(
        modifier = modifier
            .fillMaxWidth()
            .padding(16.dp)
            .background(
                color = backgroundColor,
                shape = RoundedCornerShape(12.dp)
            ),
        color = backgroundColor,
        shape = RoundedCornerShape(12.dp),
        shadowElevation = 8.dp
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.Center
        ) {
            Text(
                text = labelText,
                color = textColor,
                fontSize = 16.sp,
                fontWeight = FontWeight.Bold
            )
        }
    }
}

@Composable
fun DetailedBioloadPanel(
    band: BioloadBand,
    eegCoherence: Double,
    autonomicTone: Double,
    implantPowerRatio: Double,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(16.dp)
    ) {
        Text(
            text = "Bioload Metrics",
            fontSize = 18.sp,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.padding(bottom = 12.dp)
        )
        
        MetricRow("EEG Coherence", "%.2f%%".format(eegCoherence * 100))
        MetricRow("Autonomic Tone", "%.2f".format(autonomicTone))
        MetricRow("Implant Power Ratio", "%.1f%%".format(implantPowerRatio * 100))
        
        Spacer(modifier = Modifier.height(12.dp))
        
        // Recommendations based on band
        when (band) {
            BioloadBand.GREEN -> {
                Text(
                    text = "✓ All systems nominal. Full autonomic allowance active.",
                    color = Color(0xFF2E7D32),
                    fontSize = 12.sp
                )
            }
            BioloadBand.YELLOW -> {
                Text(
                    text = "⚠️ Biostretched-zone governance active. Reducing non-critical load. Neuroscore panel monitoring engaged.",
                    color = Color(0xFFFBC02D),
                    fontSize = 12.sp
                )
            }
            BioloadBand.RED -> {
                Text(
                    text = "🚨 EMERGENCY: All non-essential augmentation disabled. Emergency shutdown sequence initiated.",
                    color = Color(0xFFC62828),
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Bold
                )
            }
        }
    }
}

@Composable
private fun MetricRow(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(text = label, fontSize = 12.sp)
        Text(text = value, fontSize = 12.sp, fontWeight = FontWeight.Bold)
    }
}
