 Bluetooth Scanner (История)

Многоязычная реализация утилиты для сканирования Bluetooth-устройств с ведением истории обнаружений.  
Проект создан как тестовый репозиторий для демонстрации кода на 8 языках программирования.

## Особенности
- Сканирование ближайших Bluetooth-устройств (классических и BLE).
- Отображение имени устройства, MAC-адреса, уровня сигнала (RSSI) и списка сервисов (для BLE).
- Сохранение истории сканирований в JSON-файл с меткой времени.
- Возможность фильтрации устройств по имени (регулярное выражение).
- Настраиваемое время сканирования (таймаут).
- Поддержка экспорта истории в CSV.
- Кроссплатформенность (Windows, Linux, macOS).

## Установка и запуск
Каждый язык имеет свой способ сборки/запуска. Подробные инструкции приведены в комментариях к коду и ниже.

### Общие требования
- Для работы с Bluetooth необходим адаптер и соответствующие драйверы.
- На Linux может потребоваться установка `bluez` и прав доступа (`sudo`).

### Запуск на разных языках

1. **Python**  
   Установите зависимости: `pip install bleak`.  
   Запуск: `python bluetooth_scanner.py`

2. **JavaScript (Node.js)**  
   Установите `npm install @abandonware/noble`.  
   Запуск: `node bluetooth_scanner.js`

3. **Go**  
   Установите модуль: `go get github.com/go-ble/ble`.  
   Запуск: `go run bluetooth_scanner.go`

4. **Rust**  
   Добавьте `btleplug` в `Cargo.toml`.  
   Запуск: `cargo run --release`

5. **Java**  
   Используйте BlueCove (добавьте JAR в classpath).  
   Сборка: `javac -cp bluecove.jar BluetoothScanner.java`  
   Запуск: `java -cp .;bluecove.jar BluetoothScanner`

6. **C# (.NET Core)**  
   Установите пакет `InTheHand.Net.Bluetooth`.  
   Запуск: `dotnet run`

7. **C++ (Linux)**  
   Требуется BlueZ и libbluetooth-dev.  
   Сборка: `g++ -o bluetooth_scanner bluetooth_scanner.cpp -lbluetooth`  
   Запуск: `sudo ./bluetooth_scanner`

8. **Kotlin (JVM)**  
   Аналогично Java, используйте BlueCove.  
   Сборка: `kotlinc -cp bluecove.jar BluetoothScanner.kt`  
   Запуск: `kotlin -cp .;bluecove.jar BluetoothScannerKt`

## Использование
Все программы поддерживают одинаковые аргументы командной строки (где применимо):

- `--timeout <сек>` – время сканирования (по умолчанию 10).
- `--filter <regex>` – фильтр по имени устройства.
- `--history <файл>` – путь к файлу истории (JSON).
- `--export <файл.csv>` – экспортировать историю в CSV.

Пример (Python):
```bash
python bluetooth_scanner.py --timeout 15 --filter "MyDevice" --history history.json --export out.csv
Структура репозитория
text
/
├── README.md
├── bluetooth_scanner.py
├── bluetooth_scanner.js
├── bluetooth_scanner.go
├── bluetooth_scanner.rs
├── BluetoothScanner.java
├── BluetoothScanner.cs
├── bluetooth_scanner.cpp
└── BluetoothScanner.kt
Лицензия
MIT
