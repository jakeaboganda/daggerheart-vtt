#!/bin/bash
# Daggerheart VTT Demo Runner

set -e

PHASE="${1:-phase1}"

# Get local IP
LOCAL_IP=$(hostname -I | awk '{print $1}')
if [ -z "$LOCAL_IP" ]; then
    LOCAL_IP="localhost"
fi

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
        echo "✅ Server starting..."
        echo ""
        echo "📡 Access from your network:"
        echo "   🖥️  TV View:     http://$LOCAL_IP:3000"
        echo "   📱 Mobile View: http://$LOCAL_IP:3000/mobile"
        echo ""
        echo "💡 Open TV view on your browser, then scan QR code with phone!"
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
