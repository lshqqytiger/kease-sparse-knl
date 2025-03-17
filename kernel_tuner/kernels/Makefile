CXX      = icc
CXXFLAGS = -fomit-frame-pointer -O3 -W -Wall -pedantic -fopenmp -fPIC -march=knl -wd144

OBJDIR = builds
SRCDIR = src
OBJS := $(OBJDIR)/main.o
OBJS += $(OBJDIR)/compute_spmv.o
OBJS += $(OBJDIR)/compute_sptrsv.o
OBJS += $(OBJDIR)/compute_symgs.o
OBJS += $(OBJDIR)/SparseMatrix.o
OBJS += $(OBJDIR)/Vector.o
OBJS += $(OBJDIR)/CGData.o
OBJS += $(OBJDIR)/MGData.o

build: $(OBJS) $(OBJDIR)/flops

libspmv: $(OBJDIR)/libspmv.so

libsptrsv: $(OBJDIR)/libsptrsv.so

libsymgs: $(OBJDIR)/libsymgs.so

$(OBJDIR)/libspmv.so: $(SRCDIR)/spmv.cpp
	$(CXX) -shared $< -o $@ $(CXXFLAGS)

$(OBJDIR)/libsptrsv.so: $(SRCDIR)/sptrsv.cpp
	$(CXX) -shared $< -o $@ $(CXXFLAGS)

$(OBJDIR)/libsymgs.so: $(SRCDIR)/symgs.cpp
	$(CXX) -shared $< -o $@ $(CXXFLAGS)

$(OBJDIR)/%.o: $(SRCDIR)/%.cpp
	$(CXX) -c $< -o $@ $(CXXFLAGS) -ldl -lnuma

$(OBJDIR)/flops: $(OBJS)
	$(CXX) -o $@ $^ $(CXXFLAGS) -ldl -lnuma

clean:
	rm -f $(OBJDIR)/*.o $(OBJDIR)/flops $(OBJDIR)/*.so
