if [ ! -d "./libtorch" ]
then
    wget https://download.pytorch.org/libtorch/cu117/libtorch-cxx11-abi-shared-with-deps-2.0.0%2Bcu117.zip
    unzip libtorch-cxx11-abi-shared-with-deps-2.0.0+cu117.zip
fi

export LIBTORCH=$(pwd)/libtorch
export LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH