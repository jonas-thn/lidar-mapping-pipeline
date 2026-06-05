#include <stdio.h>
#include <string.h>

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_system.h"
#include "esp_wifi.h"
#include "esp_event.h"
#include "esp_log.h"
#include "nvs_flash.h"
#include "driver/uart.h"
#include "driver/gpio.h"
#include "lwip/sockets.h"
#include <arpa/inet.h>

static const char *TAG = "DATA_BRIDGE";

static void wifi_event_handler(void *arg, esp_event_base_t event_base, int32_t event_id, void *event_data)
{
    if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_START)
    {
        esp_wifi_connect();
    }
    else if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_DISCONNECTED)
    {
        ESP_LOGW(TAG, "WIFI disconnected, retry...");
        esp_wifi_connect();
    }
    else if (event_base == IP_EVENT && event_id == IP_EVENT_STA_GOT_IP)
    {
        ip_event_got_ip_t *event = (ip_event_got_ip_t *)event_data;
        ESP_LOGI(TAG, "CONNECTED! ESP32 IP: " IPSTR, IP2STR(&event->ip_info.ip));
    }
}

static void udp_stream_task(void *pvParameters)
{
    const int uart_num = UART_NUM_2;

    uart_config_t uart_config = {
        .baud_rate = 115200,
        .data_bits = UART_DATA_8_BITS,
        .parity = UART_PARITY_DISABLE,
        .stop_bits = UART_STOP_BITS_1,
        .flow_ctrl = UART_HW_FLOWCTRL_DISABLE,
        .source_clk = UART_SCLK_DEFAULT};

    ESP_ERROR_CHECK(uart_param_config(uart_num, &uart_config));
    ESP_ERROR_CHECK(uart_set_pin(uart_num, UART_PIN_NO_CHANGE, CONFIG_UART_RX_PIN, UART_PIN_NO_CHANGE, UART_PIN_NO_CHANGE));
    ESP_ERROR_CHECK(uart_driver_install(uart_num, 1024, 0, 0, NULL, 0));
    ESP_LOGI(TAG, "UART Pin %d Init", CONFIG_UART_RX_PIN);

    int sock = socket(AF_INET, SOCK_DGRAM, IPPROTO_IP);
    if (sock < 0)
    {
        ESP_LOGE(TAG, "Socket Error");
        vTaskDelete(NULL);
        return;
    }

    struct sockaddr_in dest_addr;
    dest_addr.sin_family = AF_INET;
    dest_addr.sin_port = htons(CONFIG_PC_TARGET_PORT);

    if (inet_pton(AF_INET, CONFIG_PC_TARGET_IP, &dest_addr.sin_addr) <= 0)
    {
        ESP_LOGE(TAG, "Error: Unknown IP");
        vTaskDelete(NULL);
        return;
    }

    ESP_LOGI(TAG, "UDP open: %s:%d", CONFIG_PC_TARGET_IP, CONFIG_PC_TARGET_PORT);

    uint8_t rx_buffer[256];
    uint32_t packet_counter = 0;

    while (1)
    {
        int rx_bytes = uart_read_bytes(uart_num, rx_buffer, sizeof(rx_buffer), pdMS_TO_TICKS(10));

        if (rx_bytes > 0)
        {
            packet_counter++;
            
            if (packet_counter % 20 == 0)
            {
                ESP_LOGI(TAG, "RX Heartbeat: %d Bytes vom STM32 gelesen und gesendet.", rx_bytes);
            }

            sendto(sock, rx_buffer, rx_bytes, 0, (struct sockaddr *)&dest_addr, sizeof(dest_addr));
        }
    }
}

void app_main(void)
{
    esp_err_t ret = nvs_flash_init();

    if (ret == ESP_ERR_NVS_NO_FREE_PAGES || ret == ESP_ERR_NVS_NEW_VERSION_FOUND)
    {
        ESP_ERROR_CHECK(nvs_flash_erase());
        ret = nvs_flash_init();
    }
    ESP_ERROR_CHECK(ret);

    ESP_ERROR_CHECK(esp_netif_init());
    ESP_ERROR_CHECK(esp_event_loop_create_default());
    esp_netif_create_default_wifi_sta();

    wifi_init_config_t cfg = WIFI_INIT_CONFIG_DEFAULT();
    ESP_ERROR_CHECK(esp_wifi_init(&cfg));

    ESP_ERROR_CHECK(esp_event_handler_instance_register(WIFI_EVENT, ESP_EVENT_ANY_ID, &wifi_event_handler, NULL, NULL));
    ESP_ERROR_CHECK(esp_event_handler_instance_register(IP_EVENT, IP_EVENT_STA_GOT_IP, &wifi_event_handler, NULL, NULL));

    wifi_config_t wifi_config = {
        .sta = {
            .ssid = CONFIG_WIFI_SSID,
            .password = CONFIG_WIFI_PASSWORD,
            .threshold.authmode = WIFI_AUTH_WPA2_PSK},
    };

    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_STA));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, &wifi_config));
    ESP_ERROR_CHECK(esp_wifi_start());

    xTaskCreatePinnedToCore(udp_stream_task, "udp_stream_task", 4096, NULL, 5, NULL, 1);
}
