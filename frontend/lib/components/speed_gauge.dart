import 'dart:math';
import 'package:flutter/material.dart';

class SpeedGauge extends StatelessWidget {
  final double speed;
  final double maxSpeed;
  final double size;

  const SpeedGauge(
      {super.key,
      required this.speed,
      required this.maxSpeed,
      required this.size});

  @override
  Widget build(BuildContext context) {
    return Align(
      alignment: Alignment.bottomLeft,
      child: Padding(
        padding: const EdgeInsets.only(bottom: 35, left: 8),
        child: SizedBox(
          height: size,
          width: size,
          child: Stack(
            alignment: Alignment.center,
            children: [
              Container(
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color:
                      Colors.black.withOpacity(0.2), // Half-transparent color
                ),
              ),
              Transform.rotate(
                angle: -pi,
                child: SizedBox(
                  height: size - 8,
                  width: size - 8,
                  child: CircularProgressIndicator(
                    color: Colors.pink,
                    value: speed /
                        maxSpeed, // Assuming the speed ranges from 0 to maxSpeed km/h
                    strokeWidth: 8.0,
                  ),
                ),
              ),
              Center(
                child: Text(
                  '${speed.toStringAsFixed(1)}\nkm/h',
                  textAlign: TextAlign.center,
                  style: const TextStyle(
                    height: 1.0,
                    fontSize: 12,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
