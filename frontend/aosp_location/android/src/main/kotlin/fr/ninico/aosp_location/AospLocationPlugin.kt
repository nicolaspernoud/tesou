package fr.ninico.aosp_location

import android.content.Context
import android.location.Location
import android.location.LocationListener
import android.location.LocationManager
import android.os.BatteryManager
import android.os.Build.VERSION
import android.os.Build.VERSION_CODES
import android.os.Handler
import android.os.Looper
import android.telephony.CellIdentityCdma
import android.telephony.CellIdentityGsm
import android.telephony.CellIdentityLte
import android.telephony.CellIdentityNr
import android.telephony.CellIdentityTdscdma
import android.telephony.CellIdentityWcdma
import android.telephony.CellInfo
import android.telephony.TelephonyManager
import androidx.annotation.NonNull
import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.plugin.common.EventChannel
import io.flutter.plugin.common.EventChannel.StreamHandler
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import io.flutter.plugin.common.MethodChannel.MethodCallHandler
import io.flutter.plugin.common.MethodChannel.Result
import java.util.Calendar

const val CELL_INFO_ERROR = "CELL_INFO_ERROR"
const val GPS_LOCATION_ERROR = "GPS_LOCATION_ERROR"
const val GPS_TIMEOUT_MS = 30000L

/** AospLocationPlugin */
class AospLocationPlugin : FlutterPlugin, MethodCallHandler, EventChannel.StreamHandler {
  private lateinit var channel: MethodChannel
  private lateinit var context: Context
  private var messageChannel: EventChannel? = null
  private var eventSink: EventChannel.EventSink? = null
  private lateinit var locationManager: LocationManager
  val streamLocationListener: LocationListener =
      object : LocationListener {
        override fun onLocationChanged(location: Location) {
          eventSink?.success(
              "" + location.latitude + ":" + location.longitude + ":" + getBatteryLevel().toString()
          )
        }
      }

  override fun onListen(arguments: Any?, eventSink: EventChannel.EventSink?) {
    this.eventSink = eventSink
    locationManager.requestLocationUpdates(
        LocationManager.GPS_PROVIDER,
        2500L,
        0f,
        streamLocationListener,
        Looper.myLooper()
    )
  }

  override fun onCancel(arguments: Any?) {
    locationManager.removeUpdates(streamLocationListener)
    eventSink = null
    messageChannel = null
  }

  override fun onAttachedToEngine(
      @NonNull flutterPluginBinding: FlutterPlugin.FlutterPluginBinding
  ) {
    channel = MethodChannel(flutterPluginBinding.binaryMessenger, "aosp_location")
    channel.setMethodCallHandler(this)
    this.context = flutterPluginBinding.applicationContext

    messageChannel = EventChannel(flutterPluginBinding.binaryMessenger, "aosp_location_stream")
    messageChannel?.setStreamHandler(this)
    this.locationManager = context.getSystemService(Context.LOCATION_SERVICE) as LocationManager
  }

  override fun onMethodCall(@NonNull call: MethodCall, @NonNull result: Result) {
    when (call.method) {
      "getCellInfo" -> getCellInfo(result)
      "getPositionFromGPS" -> getPositionFromGPS(result)
      else -> result.notImplemented()
    }
  }

  override fun onDetachedFromEngine(@NonNull binding: FlutterPlugin.FlutterPluginBinding) {
    channel.setMethodCallHandler(null)
    onCancel(null)
  }

  private fun getPositionFromGPS(result: MethodChannel.Result) {
    try {
      val location = locationManager.getLastKnownLocation(LocationManager.GPS_PROVIDER)
      if (location != null &&
              location.getTime() > Calendar.getInstance().getTimeInMillis() - 10 * 1000
      ) {
        result.success(
            "" + location.latitude + ":" + location.longitude + ":" + getBatteryLevel().toString()
        )
      } else {
        var replied = false
        val locationListener: LocationListener =
            object : LocationListener {
              override fun onLocationChanged(location: Location) {
                locationManager.removeUpdates(this)
                replied = true
                result.success(
                    "" +
                        location.latitude +
                        ":" +
                        location.longitude +
                        ":" +
                        getBatteryLevel().toString()
                )
              }
            }
        val myLooper = Looper.myLooper()
        locationManager.requestLocationUpdates(
            LocationManager.GPS_PROVIDER,
            5000L,
            0f,
            locationListener,
            myLooper
        )
        if (myLooper != null) {
          val myHandler = Handler(myLooper)
          myHandler.postDelayed(
              Runnable() {
                run() {
                  if (!replied) {
                    locationManager.removeUpdates(locationListener)
                    result.error(GPS_LOCATION_ERROR, "GPS Timeout", null)
                  }
                }
              },
              GPS_TIMEOUT_MS
          )
        }
      }
      ///////////////////////////////////
      // TO BE TRIED WITH ANDROID 12+ //
      /////////////////////////////////
      /*val locationRequest =
          LocationRequest.Builder(20000).setDurationMillis(GPS_TIMEOUT_MS).setMaxUpdates(1).build()
      locationManager!!.getCurrentLocation(
          LocationManager.GPS_PROVIDER,
          locationRequest,
          null,
          context.getMainExecutor(),
          Consumer { location ->
            if (location == null) result.error(GPS_LOCATION_ERROR, "location is null", null)
            else
                result.success(
                    "" +
                        location.latitude +
                        ":" +
                        location.longitude +
                        ":" +
                        getBatteryLevel().toString()
                )
          }
      )*/
      /////////////////////////////////
    } catch (ex: Exception) {
      result.error(GPS_LOCATION_ERROR, ex.message, null)
    }
  }

  private fun getCellInfo(result: MethodChannel.Result) {
    if (VERSION.SDK_INT >= VERSION_CODES.LOLLIPOP) {
      try {
        val telephonyManager =
            context.getSystemService(Context.TELEPHONY_SERVICE) as TelephonyManager?
        telephonyManager!!.requestCellInfoUpdate(
            context.getMainExecutor(),
            object : TelephonyManager.CellInfoCallback() {
              override fun onCellInfo(cellInfos: MutableList<CellInfo>) {
                if (cellInfos.size > 0) {
                  val info = cellInfos[0].getCellIdentity()
                  var type = "no_type"
                  var mcc = "no_mcc"
                  var mnc = "no_mnc"
                  var cid: Long = -1
                  var lac = -1
                  var lat = -1
                  var long = -1
                  when (info) {
                    is CellIdentityCdma -> {
                      type = "CDMA"
                      lat = info.getLatitude()
                      long = info.getLongitude()
                    }
                    is CellIdentityGsm -> {
                      type = "GSM"
                      mcc = info.getMccString() ?: mcc
                      mnc = info.getMncString() ?: mnc
                      cid = info.getCid().toLong()
                      lac = info.getLac()
                    }
                    is CellIdentityLte -> {
                      type = "LTE"
                      mcc = info.getMccString() ?: mcc
                      mnc = info.getMncString() ?: mnc
                      cid = info.getCi().toLong()
                      lac = info.getTac()
                    }
                    is CellIdentityNr -> {
                      type = "NR"
                      mcc = info.getMccString() ?: mcc
                      mnc = info.getMncString() ?: mnc
                      cid = info.getNci()
                      lac = info.getTac()
                    }
                    is CellIdentityTdscdma -> {
                      type = "TDSCDMA"
                      mcc = info.getMccString() ?: mcc
                      mnc = info.getMncString() ?: mnc
                      cid = info.getCid().toLong()
                      lac = info.getLac()
                    }
                    is CellIdentityWcdma -> {
                      type = "WCDMA"
                      mcc = info.getMccString() ?: mcc
                      mnc = info.getMncString() ?: mnc
                      cid = info.getCid().toLong()
                      lac = info.getLac()
                    }
                  }
                  val CellInfoJson =
                      """
                        {
                          "network_type": "$type",
                          "mcc": "$mcc",
                          "mnc": "$mnc",
                          "cid": $cid,
                          "lac": $lac,
                          "lat": $lat,
                          "long": $long,
                          "battery_level": ${getBatteryLevel().toString()}
                        }
                      """
                  result.success(CellInfoJson)
                } else {
                  result.error(CELL_INFO_ERROR, "empty cell info data", null)
                }
              }
              override fun onError(errorCode: Int, detail: Throwable?) {
                super.onError(errorCode, detail)
                result.error(CELL_INFO_ERROR, detail?.message, null)
              }
            }
        )
      } catch (ex: Exception) {
        result.error(CELL_INFO_ERROR, ex.message, null)
      }
    } else {
      result.error(CELL_INFO_ERROR, "android version not supported", null)
    }
  }

  private fun getBatteryLevel(): Int {
    val batteryManager = context.getSystemService(Context.BATTERY_SERVICE) as BatteryManager
    return batteryManager.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY)
  }
}
