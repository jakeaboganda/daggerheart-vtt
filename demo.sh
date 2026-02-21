#!/bin/bash
# Daggerheart VTT Demo Runner

set -e

PHASE="${1:-phase1}"

echo "🎲 Daggerheart VTT Demo"
echo "========================"
echo "Running: $PHASE"
echo ""

case "$PHASE" in
    phase1)
        echo "📋 Phase 1: Foundation & Connection"
        echo ""
        echo "Starting server..."
        cd server
        
        # Build and run server
        cargo build --release 2>&1 | grep -v "Compiling\|Finished" || true
        echo ""
        echo "✅ Server starting on http://localhost:3000"
        echo ""
        echo "🖥️  TV View:     http://localhost:3000"
        echo "📱 Mobile View: http://localhost:3000/mobile"
        echo ""
        echo "Press Ctrl+C to stop the server"
        echo ""
        
        cargo run --release
        ;;
    
    phase2)
        echo "📋 Phase 2: Basic Map & Movement"
        echo "⚠️  Not implemented yet"
        exit 1
        ;;
    
    phase3)
        echo "📋 Phase 3: Daggerheart Integration"
        echo "⚠️  Not implemented yet"
        exit 1
        ;;
    
    phase4)
        echo "📋 Phase 4: Save/Load & GM Controls"
        echo "⚠️  Not implemented yet"
        exit 1
        ;;
    
    *)
        echo "❌ Unknown phase: $PHASE"
        echo ""
        echo "Usage: ./demo.sh [phase1|phase2|phase3|phase4]"
        exit 1
        ;;
esac
