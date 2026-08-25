// BluetoothScanner.kt
import java.util.*
import javax.bluetooth.*
import javax.microedition.io.Connector
import kotlin.concurrent.thread
import kotlin.system.exitProcess
import java.io.File
import java.time.Instant
import kotlin.text.Regex

data class DeviceInfo(
    val address: String,
    val name: String,
    val rssi: Int,
    val timestamp: String,
    val services: List<String>
)

class BluetoothScanner(
    private val timeout: Int,
    private val filterPattern: String?,
    private val historyFile: String?,
    private val exportFile: String?
) : DiscoveryListener {

    private val filter = filterPattern?.let { Regex(it) }
    private val devices = mutableListOf<DeviceInfo>()
    private val history = mutableListOf<DeviceInfo>()
    private val lock = Object()
    private var finished = false

    init {
        loadHistory()
    }

    private fun loadHistory() {
        historyFile?.let { file ->
            val f = File(file)
            if (f.exists()) {
                // parse JSON (use kotlinx.serialization or manual)
            }
        }
    }

    private fun saveHistory() {
        historyFile?.let { file ->
            // serialize history to JSON
        }
    }

    private fun matchesFilter(name: String?): Boolean {
        return filter == null || (name != null && filter.containsMatchIn(name))
    }

    fun startScan() {
        val local = LocalDevice.getLocalDevice()
        val agent = local.discoveryAgent
        println("Scanning for $timeout seconds...")
        synchronized(lock) {
            agent.startInquiry(DiscoveryAgent.GIAC, this)
            try {
                lock.wait((timeout * 1000L))
            } catch (e: InterruptedException) {}
            agent.cancelInquiry(this)
        }
        finished = true
        println("Found ${devices.size} devices:")
        devices.forEach {
            println("  ${it.name} (${it.address}) RSSI: ${it.rssi} dBm")
        }
        if (historyFile != null) {
            history.addAll(devices)
            saveHistory()
            println("History updated.")
        }
    }

    override fun deviceDiscovered(btDevice: RemoteDevice, cod: DeviceClass) {
        try {
            val name = btDevice.getFriendlyName(false) ?: "Unknown"
            if (matchesFilter(name)) {
                val info = DeviceInfo(
                    address = btDevice.bluetoothAddress,
                    name = name,
                    rssi = 0, // not available
                    timestamp = Instant.now().toString(),
                    services = emptyList()
                )
                devices.add(info)
            }
        } catch (e: Exception) {}
    }

    override fun inquiryCompleted(discType: Int) {
        synchronized(lock) { lock.notify() }
    }
    override fun serviceSearchCompleted(transID: Int, respCode: Int) {}
    override fun servicesDiscovered(transID: Int, servRecord: Array<ServiceRecord>) {}

    companion object {
        @JvmStatic
        fun main(args: Array<String>) {
            var timeout = 10
            var filter: String? = null
            var history: String? = null
            var export: String? = null
            var i = 0
            while (i < args.size) {
                when (args[i]) {
                    "--timeout" -> timeout = args[++i].toInt()
                    "--filter" -> filter = args[++i]
                    "--history" -> history = args[++i]
                    "--export" -> export = args[++i]
                }
                i++
            }
            val scanner = BluetoothScanner(timeout, filter, history, export)
            scanner.startScan()
            // Export CSV if needed
        }
    }
}
