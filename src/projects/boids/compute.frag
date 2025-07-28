precision mediump float;
 
uniform sampler2D u_input_0;
uniform vec2 u_dimensions;

uniform vec2 u_space;
uniform float u_cohesion;
uniform float u_separation;
uniform float u_alignment;
uniform float u_edge_avoidance;
uniform float u_avoidance_radius;
uniform float u_detection_radius;
uniform float u_min_velocity;
uniform float u_max_velocity;
uniform float u_max_acceleration;

void main() {
   vec4 data = texture2D(u_input_0, gl_FragCoord.xy / u_dimensions);
   int max = 100; // TODO: make this a uniform
   int myIndex = int(floor(gl_FragCoord.y) * u_dimensions.x
                  + floor(gl_FragCoord.x));
   // float angle = atan(data.w, data.z);


   int num_friends = 0;
   int num_avoid = 0;
   vec2 cohesion = vec2(0);
   vec2 alignment = vec2(0);
   vec2 separation = vec2(0);


   for (int i = 0; i < 10000; ++i) {
      if (i < max) {
         float fi = float(i);
         float yIndex = floor(fi / u_dimensions.x) / u_dimensions.y + 0.5;
         float xIndex = mod(fi, u_dimensions.x) / u_dimensions.x + 0.5;

         vec4 other = texture2D(u_input_0, vec2(xIndex, yIndex));
         if(other != data) {

            float distance = length(other.xy - data.xy);
            if(distance < u_detection_radius) {
               num_friends += 1;
               cohesion += other.xy;
               //TODO: Zero case
               alignment += normalize(other.wz);
               if(distance < u_avoidance_radius) {
                  num_avoid += 1;
                  separation += data.xy - other.xy;
               }
            }
         }
      }
   }

   vec2 cohesionVel = vec2(0);
   vec2 alignmentVel = vec2(0);
   vec2 separationVel = vec2(0);

   if(num_friends > 0) {
      cohesion = cohesion / float(num_friends);
      cohesionVel = normalize(cohesion - data.xy) * u_cohesion;

      alignment = alignment / float(num_friends);
      alignmentVel = normalize(alignment) * u_alignment;

      if(length(separation) > 0.0) {
         separationVel += normalize(separation) * u_separation;
      }
   }

   data.wz = (data.wz + alignmentVel + cohesionVel + separationVel);

   if(length(data.xy) > 0.95) {
      data.wz += normalize(-data.xy) * u_edge_avoidance;
   }

   if(length(data.wz) > u_max_velocity) {
      data.wz = normalize(data.wz) * u_max_velocity;
   }

   if(length(data.wz) == 0.0){
      data.wz = vec2(0.0001);
   }

   data.xy += data.wz;
   
   gl_FragColor = data;
}
