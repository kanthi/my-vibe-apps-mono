package com.khoop.widget

import androidx.glance.appwidget.GlanceAppWidget
import androidx.glance.appwidget.GlanceAppWidgetReceiver

/** Manifest entry point for the home-screen widget — all rendering lives in [KhoopGlanceWidget]. */
class KhoopWidgetReceiver : GlanceAppWidgetReceiver() {
    override val glanceAppWidget: GlanceAppWidget = KhoopGlanceWidget()
}
