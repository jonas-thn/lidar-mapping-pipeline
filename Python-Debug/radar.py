import serial
import pygame
import sys

SERIAL_PORT = 'COM5'  
BAUD_RATE = 115200
SCALE = 0.25          
                      
pygame.init()
WIDTH, HEIGHT = 800, 800
screen = pygame.display.set_mode((WIDTH, HEIGHT))
pygame.display.set_caption("LiDAR Echtzeit Radar")
clock = pygame.time.Clock()

fade_surface = pygame.Surface((WIDTH, HEIGHT))
fade_surface.fill((0, 0, 0))
fade_surface.set_alpha(5)  

try:
    ser = serial.Serial(SERIAL_PORT, BAUD_RATE, timeout=0)
except Exception as e:
    print(f"Error: {e}")
    sys.exit()

center_x, center_y = WIDTH // 2, HEIGHT // 2
buffer = b''

while True:
    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            ser.close()
            pygame.quit()
            sys.exit()

    buffer += ser.read(ser.in_waiting)
    
    if b'\n' in buffer:
        lines = buffer.split(b'\n')
        buffer = lines[-1]  
        
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