Integrate GraphOpEvent replay system with the enhanced 3D graph visualization in log-viewer:

**Problem**: The log-viewer currently parses GraphOpEvent logs but doesn't fully leverage the Graph3D component's capabilities for algorithm visualization. The GraphOpEvent system in `context-trace` provides structured logging of graph operations (insert, search, delete, etc.) that should be visualized in 3D.

**Integration Requirements:**
1. **GraphOpEvent parsing**: Enhance log-viewer to parse GraphOpEvent JSON format from context-trace
2. **3D visualization**: Use Graph3D component to visualize algorithm execution as animated graph operations
3. **Time-based replay**: Add playback controls (play, pause, step, rewind) for algorithm visualization
4. **Visual enhancements**: 
   - Highlight active nodes/edges during operations
   - Show operation type (insert, search, delete) with visual cues
   - Use property-based rendering tiers for focus during replay
   - Enable Fixed2D mode for algorithm presentation

**Technical Implementation:**
- Update `log-viewer/app.rs` to parse GraphOpEvent format
- Create `GraphReplayState` struct to manage playback state
- Integrate with Graph3D component's animation capabilities
- Add playback UI controls (timeline, speed, step controls)
- Use Graph3D's keyframing for smooth transitions between states

**Acceptance Criteria:**
1. log-viewer can parse and visualize GraphOpEvent logs
2. Algorithm replay works with play/pause/step controls
3. Graph operations are visually distinct (insert vs search vs delete)
4. Property-based rendering tiers work during replay
5. Fixed2D mode provides clear algorithm presentation
6. Performance is acceptable for typical algorithm traces