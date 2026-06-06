#py -3.12 .\radar-udp.py

import socket
import pygame
import sys
import struct 

UDP_IP = "0.0.0.0" 
UDP_PORT = 4242     
SCALE = 0.25
                      
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.bind((UDP_IP, UDP_PORT))
sock.setblocking(False) 

pygame.init()
WIDTH, HEIGHT = 800, 800
screen = pygame.display.set_mode((WIDTH, HEIGHT))
pygame.display.set_caption("LiDAR Binary Telemetry")
clock = pygame.time.Clock()

fade_surface = pygame.Surface((WIDTH, HEIGHT))
fade_surface.fill((0, 0, 0))
fade_surface.set_alpha(5)  

center_x, center_y = WIDTH // 2, HEIGHT // 2
buffer = bytearray()

while True:
    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            sock.close()
            pygame.quit()
            sys.exit()

    try:
        while True:
            data, addr = sock.recvfrom(4096)
            buffer.extend(data)
    except BlockingIOError:
        pass 
    
    while len(buffer) >= 8:
        if buffer[0] == 0x55 and buffer[1] == 0xAA:
            packet = buffer[:8]
            del buffer[:8]
            
            try:
                sync, x_mm, y_mm, z_mm = struct.unpack('<Hhhh', packet)
                
                px = center_x + int(x_mm * SCALE)
                py = center_y - int(y_mm * SCALE)
                
                if 0 <= px < WIDTH and 0 <= py < HEIGHT:
                    pygame.draw.circle(screen, (0, 255, 255), (px, py), 2)
            except struct.error:
                pass
        else:
            del buffer[:1]

    screen.blit(fade_surface, (0, 0))
    pygame.draw.circle(screen, (255, 0, 0), (center_x, center_y), 4)
    
    pygame.display.flip()
    clock.tick(60)