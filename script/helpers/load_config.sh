arch=$(cat .config/arch)
board=$(cat .config/board)

source "script/config/arch_$arch.sh"
source "script/config/board_$board.sh"