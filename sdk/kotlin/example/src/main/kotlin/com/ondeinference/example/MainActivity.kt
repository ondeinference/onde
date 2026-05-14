package com.ondeinference.example

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import com.ondeinference.example.ui.ChatScreen
import com.ondeinference.example.ui.theme.OndeExampleTheme

class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Draw behind the status and nav bars so the layout owns the full screen.
        enableEdgeToEdge()

        setContent {
            OndeExampleTheme {
                ChatScreen()
            }
        }
    }
}
