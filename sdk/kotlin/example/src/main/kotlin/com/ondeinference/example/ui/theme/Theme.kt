package com.ondeinference.example.ui.theme

import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext

// On Android 12+ we pick up the user's wallpaper colours automatically.
// On older devices we fall back to this hand-picked teal palette.

private val FallbackLightScheme = lightColorScheme(
    primary              = Color(0xFF006874),
    onPrimary            = Color(0xFFFFFFFF),
    primaryContainer     = Color(0xFF97F0FF),
    onPrimaryContainer   = Color(0xFF001F24),
    secondary            = Color(0xFF4A6267),
    onSecondary          = Color(0xFFFFFFFF),
    secondaryContainer   = Color(0xFFCDE7EC),
    onSecondaryContainer = Color(0xFF051F23),
    tertiary             = Color(0xFF525E7D),
    onTertiary           = Color(0xFFFFFFFF),
    tertiaryContainer    = Color(0xFFD9E2FF),
    onTertiaryContainer  = Color(0xFF0E1B37),
    error                = Color(0xFFBA1A1A),
    onError              = Color(0xFFFFFFFF),
    errorContainer       = Color(0xFFFFDAD6),
    onErrorContainer     = Color(0xFF410002),
    background           = Color(0xFFFAFDFD),
    onBackground         = Color(0xFF191C1D),
    surface              = Color(0xFFFAFDFD),
    onSurface            = Color(0xFF191C1D),
    surfaceVariant       = Color(0xFFDBE4E6),
    onSurfaceVariant     = Color(0xFF3F484A),
)

private val FallbackDarkScheme = darkColorScheme(
    primary              = Color(0xFF4FD8EB),
    onPrimary            = Color(0xFF00363D),
    primaryContainer     = Color(0xFF004F58),
    onPrimaryContainer   = Color(0xFF97F0FF),
    secondary            = Color(0xFFB1CBD0),
    onSecondary          = Color(0xFF1C3438),
    secondaryContainer   = Color(0xFF334B4F),
    onSecondaryContainer = Color(0xFFCDE7EC),
    tertiary             = Color(0xFFBAC6E8),
    onTertiary           = Color(0xFF24304D),
    tertiaryContainer    = Color(0xFF3A4664),
    onTertiaryContainer  = Color(0xFFD9E2FF),
    error                = Color(0xFFFFB4AB),
    onError              = Color(0xFF690005),
    errorContainer       = Color(0xFF93000A),
    onErrorContainer     = Color(0xFFFFDAD6),
    background           = Color(0xFF191C1D),
    onBackground         = Color(0xFFE1E3E3),
    surface              = Color(0xFF191C1D),
    onSurface            = Color(0xFFE1E3E3),
    surfaceVariant       = Color(0xFF3F484A),
    onSurfaceVariant     = Color(0xFFBFC8CA),
)

@Composable
fun OndeExampleTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit,
) {
    val colorScheme = when {
        Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
            val context = LocalContext.current
            if (darkTheme) dynamicDarkColorScheme(context)
            else dynamicLightColorScheme(context)
        }
        darkTheme -> FallbackDarkScheme
        else      -> FallbackLightScheme
    }

    MaterialTheme(
        colorScheme = colorScheme,
        content     = content,
    )
}
