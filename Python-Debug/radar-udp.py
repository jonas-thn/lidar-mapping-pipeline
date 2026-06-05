#py -3.12 .\radar-udp.py

import socket
import pygame
import sys

UDP_IP = "0.0.0.0" 
UDP_PORT = 4242     
SCALE = 0.25
                      
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.bind((UDP_IP, UDP_PORT))
sock.setblocking(False) 

pygame.init()
WIDTH, HEIGHT = 800, 800
screen = pygame.display.set_mode((WIDTH, HEIGHT))
pygame.display.set_caption("LiDAR Wireless Telemetry")
clock = pygame.time.Clock()

fade_surface = pygame.Surface((WIDTH, HEIGHT))
fade_surface.fill((0, 0, 0))
fade_surface.set_alpha(5)  

center_x, center_y = WIDTH // 2, HEIGHT // 2
buffer = b''

print(f"Echolot aktiv. Lausche auf UDP Port {UDP_PORT}...")

while True:
    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            sock.close()
            pygame.quit()
            sys.exit()

    try:
        while True:
            data, addr = sock.recvfrom(4096)
            buffer += data
    except BlockingIOError:
        pass 
    
    if b'\n' in buffer:
        lines = buffer.split(b'\n')
        buffer = lines[-1]  # Den unvollständigen Rest für später aufheben
        
        for line in lines[:-1]:
            try:
                x_str, y_str = line.decode('utf-8').strip().split(',')
                x_mm = int(x_str)
                y_mm = int(y_str)  
                
                px = center_x + int(x_mm * SCALE)
                py = center_y - int(y_mm * SCALE)
                
                if 0 <= px < WIDTH and 0 <= py < HEIGHT:
                    pygame.draw.circle(screen, (0, 255, 255), (px, py), 2)
            except:
                pass 

    screen.blit(fade_surface, (0, 0))
    pygame.draw.circle(screen, (255, 0, 0), (center_x, center_y), 4)
    
    pygame.display.flip()
    clock.tick(60)