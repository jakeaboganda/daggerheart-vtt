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
        echo "🗺️  Phase 2 Features:"
        echo "   ✨ 2D map with player tokens"
        echo "   🎨 Each player has unique color"
        echo "   📍 Tap mobile screen to move"
        echo "   🔄 Real-time movement sync"
        echo ""
        echo "💡 Open TV view, then join from phones and move around!"
        echo "Press Ctrl+C to stop the server"
        echo ""
        
        cargo run --release
        ;;
    
    phase3)
        echo "📋 Phase 3: Daggerheart Integration"
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
        echo "🎲 Phase 3 Features:"
        echo "   ✨ Character creation (class, ancestry, attributes)"
        echo "   📊 Character sheets on mobile"
        echo "   🎲 Dice rolling with duality system"
        echo "   📺 Roll results displayed on TV"
        echo "   ❤️  HP/Stress/Hope tracking"
        echo ""
        echo "💡 Open TV view, join from phones, create characters, and roll!"
        echo "Press Ctrl+C to stop the server"
        echo ""
        
        cargo run --release
        ;;
    
    phase4)
        echo "📋 Phase 4: Save/Load & GM Controls"
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
        echo "   🎮 GM View:     http://$LOCAL_IP:3000/gm"
        echo ""
        echo "🎮 Phase 4 Features:"
        echo "   💾 Save/Load game sessions"
        echo "   📊 GM view with full game state"
        echo "   👥 Player management panel"
        echo "   🗺️  Map view with all players"
        echo "   📝 Session history"
        echo ""
        echo "💡 Open GM view to save/load sessions!"
        echo "💡 Saves are stored in server/saves/ directory"
        echo "Press Ctrl+C to stop the server"
        echo ""
        
        cargo run --release
        ;;
    
    *)
        echo "❌ Unknown phase: $PHASE"
        echo ""
        echo "Usage: ./demo.sh [phase1|phase2|phase3|phase4]"
        exit 1
        ;;
esac
